use crate::{methods::receiver::Receiver, self_type::SelfType};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

struct SpanMarker(proc_macro2::Span);

impl Spanned for SpanMarker {
    fn span(&self) -> proc_macro2::Span {
        self.0
    }
}

pub struct Signature<'a> {
    name: &'a syn::Ident,
    arg_idents: Vec<syn::Ident>,
    is_async: bool,
    output: Option<&'a syn::Type>,
    method_data: Option<MethodData<'a>>,
    trait_path: Option<&'a syn::Path>,
}

pub struct MethodData<'a> {
    receiver: Receiver,
    arg_types: Vec<WhenArg<'a>>,
    is_private: bool,
}

#[derive(Debug)]
pub struct WhenArg<'a>(&'a syn::Type);

impl<'a> ToTokens for WhenArg<'a> {
    fn to_tokens(&self, token_stream: &mut proc_macro2::TokenStream) {
        match self.0 {
            syn::Type::ImplTrait(ty) => {
                let bounds = &ty.bounds;
                if bounds
                    .iter()
                    .any(|b| matches!(b, syn::TypeParamBound::Lifetime(_)))
                {
                    token_stream.extend(quote! { std::boxed::Box<dyn #bounds> });
                } else {
                    // avoid implicit static
                    token_stream.extend(quote! { std::boxed::Box<dyn #bounds + '_> });
                }
            }
            ty => ty.to_tokens(token_stream),
        }
    }
}

impl<'a> Signature<'a> {
    pub fn morph(
        signature: &'a mut syn::Signature,
        trait_path: Option<&'a syn::Path>,
        vis: &syn::Visibility,
    ) -> Signature<'a> {
        let is_async = signature.asyncness.is_some();
        let name = &signature.ident;
        let receiver = Receiver::from_signature(signature);

        let len = signature.inputs.len();
        let output = match &signature.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.as_ref()),
        };

        let mut method_data = receiver.map(|receiver| MethodData {
            receiver,
            is_private: trait_path.is_none() && *vis == syn::Visibility::Inherited,
            arg_types: Vec::with_capacity(len - 1),
        });

        let mut arg_idents =
            Vec::with_capacity(signature.inputs.len() - method_data.is_some() as usize);

        signature
            .inputs
            .iter_mut()
            .skip(method_data.is_some() as usize) // if it's a method; skip first arg
            .map(|a| match a {
                syn::FnArg::Typed(arg) => arg,
                syn::FnArg::Receiver(_) => {
                    unreachable!("this is a weird bug in faux if you reached this")
                }
            })
            .enumerate()
            .for_each(|(i, arg)| {
                // normalize all names.
                let ident = syn::Ident::new(&format!("_x{}", i), proc_macro2::Span::call_site());
                arg.pat = Box::new(syn::Pat::Ident(syn::PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    subpat: None,
                    ident: ident.clone(),
                }));
                arg_idents.push(ident);
                if let Some(m) = &mut method_data {
                    m.arg_types.push(WhenArg(&*arg.ty));
                }
            });

        Signature {
            is_async,
            name,
            arg_idents,
            output,
            method_data,
            trait_path,
        }
    }

    pub fn create_body(
        &self,
        real_self: SelfType,
        real_ty: &syn::TypePath,
        morphed_ty: &syn::TypePath,
    ) -> darling::Result<syn::Block> {
        let name = self.name;
        let arg_idents = &self.arg_idents;

        let proxy = match self.trait_path {
            None => quote! { <#real_ty>::#name },
            Some(path) => quote! { <#real_ty as #path>::#name },
        };

        let real_self_arg = if self.method_data.is_some() {
            // need to pass the real Self arg to the real method
            Some(syn::Ident::new("r", proc_macro2::Span::call_site()))
        } else {
            None
        };
        let proxy_args = real_self_arg.iter().chain(arg_idents);
        let mut proxy_real = quote! { #proxy(#(#proxy_args),*) };
        if self.is_async {
            proxy_real.extend(quote! { .await })
        }
        if let Some(wrapped_self) = self.wrap_self(morphed_ty, real_self, &proxy_real)? {
            proxy_real = wrapped_self;
        }

        let ret = match &self.method_data {
            // not mockable
            // proxy to real associated function
            None => syn::parse2(proxy_real).unwrap(),
            // else we can either proxy for real instances
            // or call the mock store for faux instances
            Some(method_data) => {
                let call_mock = if method_data.is_private {
                    quote! { panic!("faux error: private methods are not mockable; and therefore not directly callable in a mock") }
                } else {
                    let faux_ident =
                        syn::Ident::new(&format!("_faux_{}", name), proc_macro2::Span::call_site());

                    let mut args =
                        arg_idents
                            .iter()
                            .zip(method_data.arg_types.iter())
                            .map(|(ident, ty)| match ty.0 {
                                syn::Type::ImplTrait(ty) => {
                                    let bounds = &ty.bounds;
                                    quote! {
                                        std::boxed::Box::new(#ident) as std::boxed::Box<dyn #bounds>
                                    }
                                }
                                _ => quote! { #ident },
                            });

                    let args = if arg_idents.len() == 1 {
                        let arg = args.next().unwrap();
                        quote! { #arg }
                    } else {
                        quote! { (#(#args,)*) }
                    };

                    let struct_and_method_name =
                        format!("{}::{}", morphed_ty.to_token_stream(), name);
                    quote! {
                        unsafe {
                            match q.call_mock(<Self>::#faux_ident, #args) {
                                std::result::Result::Ok(o) => o,
                                std::result::Result::Err(e) => {
                                    panic!("failed to call mock on '{}':\n{}", #struct_and_method_name, e);
                                }
                            }
                        }
                    }
                };

                method_data
                    .receiver
                    .method_body(real_self, proxy_real, call_mock)?
            }
        };

        Ok(syn::Block {
            stmts: vec![syn::Stmt::Expr(ret)],
            brace_token: Default::default(),
        })
    }

    pub fn create_when(&self) -> Option<Vec<syn::ImplItemMethod>> {
        self.method_data
            .as_ref()
            .filter(|m| !m.is_private)
            .map(|m| m.create_when(self.output, &self.name))
    }

    fn wrap_self(
        &self,
        morphed_ty: &syn::TypePath,
        real_self: SelfType,
        block: &TokenStream,
    ) -> darling::Result<Option<TokenStream>> {
        let is_self = |ty: &syn::TypePath| {
            ty == morphed_ty || (ty.qself.is_none() && ty.path.is_ident("Self"))
        };

        let self_generic = |args: &syn::PathArguments| match args {
            syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args,
                ..
            }) if args.len() == 1 => match args.first().unwrap() {
                syn::GenericArgument::Type(syn::Type::Path(ty)) => is_self(&ty),
                _ => false,
            },
            _ => false,
        };

        Ok(if let Some(syn::Type::Path(output)) = self.output {
            let last_segment = &output.path.segments.last().unwrap();
            match SelfType::from_path(output) {
                SelfType::Owned if is_self(output) => Some(match real_self {
                    SelfType::Owned => quote! { Self(faux::MaybeFaux::Real(#block)) },
                    generic => {
                        let new_path = generic
                            .new_path()
                            .expect("Generic self should have new() function");
                        quote! { Self(faux::MaybeFaux::Real(#new_path(#block))) }
                    }
                }),
                generic if self_generic(&last_segment.arguments) => {
                    if generic == real_self {
                        let new_path = real_self
                            .new_path()
                            .expect("return type should not be Self");
                        Some(quote! { #new_path(Self(faux::MaybeFaux::Real(#block))) })
                    } else {
                        return Err(darling::Error::custom(wrong_self_type_error(
                            generic, real_self,
                        ))
                        .with_span(&output));
                    }
                }
                _ => None,
            }
        } else {
            None
        })
    }
}

impl<'a> MethodData<'a> {
    pub fn create_when(
        &self,
        output: Option<&syn::Type>,
        name: &syn::Ident,
    ) -> Vec<syn::ImplItemMethod> {
        let &MethodData {
            ref arg_types,
            ref receiver,
            ..
        } = self;
        let receiver_tokens = &receiver.tokens;

        let when_ident =
            syn::Ident::new(&format!("_when_{}", name), proc_macro2::Span::call_site());
        let faux_ident =
            syn::Ident::new(&format!("_faux_{}", name), proc_macro2::Span::call_site());

        let empty = syn::parse_quote! { () };
        let output = output.unwrap_or(&empty);

        let when_method = syn::parse_quote! {
            pub fn #when_ident(&mut self) -> faux::When<#receiver_tokens, (#(#arg_types),*), #output, faux::when::Any> {
                match &mut self.0 {
                    faux::MaybeFaux::Faux(faux) => faux::When::new(
                        <Self>::#faux_ident,
                        faux
                    ),
                    faux::MaybeFaux::Real(_) => panic!("not allowed to mock a real instance!"),
                }
            }
        };

        let faux_method = syn::parse_quote! {
            #[allow(clippy::needless_arbitrary_self_type)]
            pub fn #faux_ident(self: #receiver_tokens, input: (#(#arg_types),*)) -> #output {
                panic!("do not call this")
            }
        };

        vec![when_method, faux_method]
    }
}

fn wrong_self_type_error(expected: SelfType, received: SelfType) -> impl std::fmt::Display {
    format!(
        "faux cannot create {expected} from a self type of {received}. Consider specifying a different self_type in the faux attributes, or moving this method to a non-faux impl block",
        expected = expected,
        received = received
    )
}
