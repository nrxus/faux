use crate::{methods::receiver::Receiver, self_type::SelfType};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    spanned::Spanned, GenericArgument, Generics, Ident, Lifetime, PathArguments, PathSegment, Type,
    TypePath,
};

pub struct Signature<'a> {
    name: &'a syn::Ident,
    args: Vec<&'a syn::Pat>,
    is_async: bool,
    output: Option<&'a syn::Type>,
    method_data: Option<MethodData<'a>>,
    trait_path: Option<&'a syn::Path>,
}

pub struct MethodData<'a> {
    receiver: Receiver,
    generics: syn::Generics,
    arg_types: Vec<WhenArg<'a>>,
    is_private: bool,
}

#[derive(Debug)]
pub struct WhenArg<'a>(&'a syn::Type);

pub fn has_impl_trait(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::ImplTrait(_) => true,
        syn::Type::Reference(reference) => has_impl_trait(reference.elem.as_ref()),
        _ => false,
    }
}

pub fn replace_impl_trait(ty: &syn::Type) -> Option<syn::Type> {
    match ty {
        syn::Type::ImplTrait(ty) => {
            let mut bounds = ty.bounds.clone();

            if bounds
                .iter()
                .all(|b| !matches!(b, syn::TypeParamBound::Lifetime(_)))
            {
                bounds.push(syn::TypeParamBound::Lifetime(syn::Lifetime::new(
                    "'_",
                    proc_macro2::Span::call_site(),
                )));
            }

            let ty = syn::Type::Paren(syn::TypeParen {
                paren_token: syn::token::Paren(proc_macro2::Span::call_site()),
                elem: Box::new(syn::Type::TraitObject(syn::TypeTraitObject {
                    bounds,
                    dyn_token: Some(syn::Token![dyn](proc_macro2::Span::call_site())),
                })),
            });

            Some(ty)
        }
        syn::Type::Reference(syn::TypeReference {
            and_token,
            lifetime,
            mutability,
            elem,
        }) => replace_impl_trait(elem).map(|ty| {
            syn::Type::Reference(syn::TypeReference {
                elem: Box::new(ty),
                lifetime: lifetime.clone(),
                mutability: *mutability,
                and_token: *and_token,
            })
        }),
        _ => None,
    }
}

impl ToTokens for WhenArg<'_> {
    fn to_tokens(&self, token_stream: &mut proc_macro2::TokenStream) {
        match replace_impl_trait(self.0) {
            None => self.0.to_tokens(token_stream),
            Some(impl_ty) => {
                token_stream.extend(quote! { std::boxed::Box<#impl_ty> });
            }
        }
    }
}

impl<'a> Signature<'a> {
    pub fn morph(
        signature: &'a syn::Signature,
        trait_path: Option<&'a syn::Path>,
        vis: &syn::Visibility,
    ) -> Signature<'a> {
        let receiver = Receiver::from_signature(signature);
        let generics = signature.generics.clone();

        let output = match &signature.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.as_ref()),
        };

        let method_data = receiver.map(|receiver| {
            let arg_types = signature
                .inputs
                .iter()
                .skip(1)
                .map(|a| match a {
                    syn::FnArg::Typed(arg) => WhenArg(&arg.ty),
                    syn::FnArg::Receiver(_) => {
                        unreachable!("this is a weird bug in faux if you reached this")
                    }
                })
                .collect();

            MethodData {
                receiver,
                generics,
                arg_types,
                is_private: trait_path.is_none() && *vis == syn::Visibility::Inherited,
            }
        });

        Signature {
            name: &signature.ident,
            args: signature
                .inputs
                .iter()
                .skip(method_data.is_some() as usize)
                .map(|a| match a {
                    syn::FnArg::Typed(arg) => &*arg.pat,
                    syn::FnArg::Receiver(_) => {
                        unreachable!("this is a weird bug in faux if you reached this")
                    }
                })
                .collect(),
            is_async: signature.asyncness.is_some(),
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
        let name = &self.name;
        let args = &self.args;

        let generics = self
            .method_data
            .as_ref()
            .map(|method_data| method_data.generics.clone());

        let generic_idents = generic_type_idents(generics);
        let turbofish = turbofish(&generic_idents);

        let proxy = match self.trait_path {
            None => quote! { <#real_ty>::#name #turbofish },
            Some(path) => quote! { <#real_ty as #path>::#name #turbofish },
        };

        let real_self_arg = self.method_data.as_ref().map(|_| {
            // need to pass the real Self arg to the real method
            syn::Pat::Ident(syn::PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: syn::Ident::new("_maybe_faux_real", proc_macro2::Span::call_site()),
                subpat: None,
            })
        });
        let real_self_arg = real_self_arg.as_ref();

        let proxy_args = real_self_arg.iter().chain(args);
        let mut proxy_real = quote! { #proxy(#(#proxy_args),*) };
        if self.is_async {
            proxy_real.extend(quote! { .await })
        }
        if let Some(wrapped_self) = self.wrap_self(morphed_ty, real_self, &proxy_real)? {
            proxy_real = wrapped_self;
        }

        let ret = match &self.method_data {
            // not stubbable
            // proxy to real associated function
            None => syn::parse2(proxy_real).unwrap(),
            // else we can either proxy for real instances
            // or call the mock store for faux instances
            Some(method_data) => {
                let call_stub = if method_data.is_private {
                    quote! { panic!("faux error: private methods are not stubbable; and therefore not directly callable in a mock") }
                } else {
                    let faux_ident =
                        syn::Ident::new(&format!("_faux_{}", name), proc_macro2::Span::call_site());

                    let mut args =
                        args.iter()
                            .zip(method_data.arg_types.iter())
                            .map(|(ident, ty)| {
                                if has_impl_trait(ty.0) {
                                    quote! {
                                        std::boxed::Box::new(#ident)
                                    }
                                } else {
                                    quote! { #ident }
                                }
                            });

                    let args = if args.len() == 1 {
                        let arg = args.next().unwrap();
                        quote! { #arg }
                    } else {
                        quote! { (#(#args,)*) }
                    };

                    let fn_name = name.to_string();
                    let mut generics_str = generic_idents
                        .into_iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    generics_str.retain(|c| !c.is_whitespace());

                    quote! {
                        unsafe {
                            match _maybe_faux_faux.call_stub(<Self>::#faux_ident #turbofish, #fn_name, #args, #generics_str) {
                                std::result::Result::Ok(o) => o,
                                std::result::Result::Err(e) => panic!("{}", e),
                            }
                        }
                    }
                };

                method_data
                    .receiver
                    .method_body(real_self, proxy_real, call_stub)?
            }
        };

        Ok(syn::Block {
            stmts: vec![syn::Stmt::Expr(ret, None)],
            brace_token: Default::default(),
        })
    }

    pub fn create_when(&self) -> Option<Vec<syn::ImplItemFn>> {
        self.method_data
            .as_ref()
            .filter(|m| !m.is_private)
            .map(|m| m.create_when(self.output, self.name))
    }

        fn is_self(ty: &syn::TypePath, morphed_ty: &syn::TypePath) -> bool {
        ty == morphed_ty || (ty.qself.is_none() && ty.path.is_ident("Self"))
    }

    fn wrap_self(
        &self,
        morphed_ty: &syn::TypePath,
        real_self: SelfType,
        block: &TokenStream,
    ) -> darling::Result<Option<TokenStream>> {
        // TODO: use let-else once we bump MSRV to 1.65.0
        let output = match self.output {
            Some(output) => output,
            None => return Ok(None),
        };
        
        Self::wrap_self_specific(output, morphed_ty, real_self, block)
    }

    fn wrap_self_specific(
        ty: &Type,
        morphed_ty: &syn::TypePath,
        real_self: SelfType,
        block: &TokenStream,
    ) -> darling::Result<Option<TokenStream>> {
        if !contains_self(ty, morphed_ty) {
            return Ok(None);
        }

        let output = match ty {
            syn::Type::Path(output) => output,
            syn::Type::Tuple(tuple) => {
                return Ok(Some(Self::wrap_self_tuple(block, tuple, morphed_ty, real_self)));
            },
            output => return Err(unhandled_self_return(output)),
        };

        let wrapped = if Self::is_self(output, morphed_ty) {
            match real_self {
                SelfType::Owned => quote! { Self(faux::MaybeFaux::Real(#block)) },
                generic => {
                    let new_path = generic
                        .new_path()
                        .expect("Generic self should have new() function");
                    quote! { Self(faux::MaybeFaux::Real(#new_path(#block))) }
                }
            }
        } else {
            let unpathed_output = output.path.segments.last().unwrap();
            let generics = match &unpathed_output.arguments {
                syn::PathArguments::AngleBracketed(args) => args,
                g => return Err(unhandled_self_return(g)),
            };
            let first_arg = generics
                .args
                .first()
                .expect("faux bug: no generic arguments but expected at least one");
            let first_arg = match first_arg {
                syn::GenericArgument::Type(syn::Type::Path(ty)) => ty,
                _ => return Err(unhandled_self_return(generics)),
            };

            if !Self::is_self(first_arg, morphed_ty) {
                return Err(unhandled_self_return(generics));
            }

            let output_ident = &unpathed_output.ident;
            match real_self {
                SelfType::Rc if output_ident == "Rc" => {
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(#block))) }
                }
                SelfType::Arc if output_ident == "Arc" => {
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(#block))) }
                }
                SelfType::Box if output_ident == "Box" => {
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(#block))) }
                }
                SelfType::Owned if output_ident == "Result" || output_ident == "Option" => {
                    quote! { { #block }.map(faux::MaybeFaux::Real).map(Self) }
                }
                SelfType::Owned if output_ident == "Box" => {
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(*#block))) }
                }
                SelfType::Owned if output_ident == "Rc" || output_ident == "Arc" => {
                    let ungenerified = {
                        // clone so we don't modify the original output
                        let mut output = output.clone();
                        output.path.segments.last_mut().unwrap().arguments = PathArguments::None;
                        output
                    };
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(
                        #ungenerified::try_unwrap(#block).ok().expect("faux: failed to grab value from reference counter because it was not unique.")
                    ))) }
                }
                _ => return Err(unhandled_self_return(output)),
            }
        };

        Ok(Some(wrapped))
    }

    fn wrap_self_tuple(
        block: &TokenStream, 
        tuple: &syn::TypeTuple, 
        morphed_ty: &syn::TypePath,
        real_self: SelfType) -> TokenStream {
        let elements = tuple.elems
            .iter()
            .enumerate()
            .map(|e| {
                let index = syn::Index::from(e.0);
                let ty = e.1;

                let tuple_index = quote! { tuple.#index };
                let wrapped = Self::wrap_self_specific(ty, morphed_ty, real_self, &tuple_index);

                match wrapped {
                    Ok(Some(self_type)) => self_type,
                    Ok(None) => quote! { tuple.#index },
                    Err(_) => quote! { tuple.#index }
                }
            });
        
        quote! {{ 
            let tuple = #block;
           
            (# ( #elements),*)
        }}
    }
}

impl MethodData<'_> {
    pub fn create_when(
        &self,
        output: Option<&syn::Type>,
        name: &syn::Ident,
    ) -> Vec<syn::ImplItemFn> {
        let MethodData {
            arg_types,
            receiver,
            generics,
            ..
        } = self;
        let mut receiver_ty = receiver.ty.clone();
        add_lifetime(&mut receiver_ty, &syn::parse(quote! { 'm }.into()).unwrap());

        let arg_types = arg_types
            .iter()
            .map(|arg_type| {
                let mut typ = arg_type.0.clone();
                add_lifetime(&mut typ, &syn::parse(quote! { '_ }.into()).unwrap());
                let new_arg_type = WhenArg(&typ);
                quote! { #new_arg_type }
            })
            .collect::<Vec<_>>();

        let when_ident =
            syn::Ident::new(&format!("_when_{}", name), proc_macro2::Span::call_site());
        let faux_ident =
            syn::Ident::new(&format!("_faux_{}", name), proc_macro2::Span::call_site());

        let empty = syn::parse_quote! { () };
        let mut output = output.unwrap_or(&empty).clone();
        add_lifetime(&mut output, &syn::parse(quote! { '_ }.into()).unwrap());

        let name_str = name.to_string();

        let generic_params = &generics.params;
        let generics_contents = quote! { 'm, #generic_params };

        let generics_where_clause = &generics.where_clause;

        let generic_idents = generic_type_idents(Some(generics.clone()));
        let turbofish = turbofish(&generic_idents);

        let when_method = syn::parse_quote! {
            pub fn #when_ident<#generics_contents>(&'m mut self) -> faux::When<'m, #receiver_ty, (#(#arg_types),*), #output, faux::matcher::AnyInvocation> #generics_where_clause {
                match &mut self.0 {
                    faux::MaybeFaux::Faux(_maybe_faux_faux) => faux::When::new(
                        <Self>::#faux_ident #turbofish,
                        #name_str,
                        _maybe_faux_faux
                    ),
                    faux::MaybeFaux::Real(_) => panic!("not allowed to stub a real instance!"),
                }
            }
        };

        let proxy = quote! { <Self>::#faux_ident #turbofish };

        let panic_message = format!("do not call this ({proxy})");

        let faux_method = syn::parse_quote! {
            #[allow(clippy::needless_arbitrary_self_type)]
            #[allow(clippy::boxed_local)]
            pub fn #faux_ident <#generics_contents> (self: #receiver_ty, _: (#(#arg_types),*)) -> #output #generics_where_clause {
                panic!(concat!(#panic_message, "{:?}"), #proxy as *const ())
            }
        };

        vec![when_method, faux_method]
    }
}

// traverse a type, and add the provided lifetime to references that don't have any lifetime yet.
// Eg(pseudocode): `add_lifetime(Result<Option<&str>, ()>, 'a)`
// will return
// `Result<Option<&'a str>, ()>`
fn add_lifetime(t: &mut Type, new_lifetime: &Lifetime) {
    match t {
        Type::Reference(ref mut reference) => {
            let lifetime = &mut reference.lifetime;
            if lifetime.is_none() {
                lifetime.replace(new_lifetime.clone());
            }
        }
        Type::Path(TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => {
            if let Some(PathSegment {
                arguments: syn::PathArguments::AngleBracketed(ref mut args),
                ..
            }) = segments.last_mut()
            {
                if let Some(first_arg) = args.args.first_mut() {
                    match first_arg {
                        GenericArgument::Type(t) => add_lifetime(t, new_lifetime),
                        _ => {}
                    }
                }
            };
        }
        _ => {}
    }
}

fn unhandled_self_return(spanned: impl Spanned) -> darling::Error {
    darling::Error::custom("faux: the return type refers to the mocked struct in a way that faux cannot handle. Split this function into an `impl` block not marked by #[faux::methods]. If you believe this is a mistake or it's a case that should be handled by faux please file an issue").with_span(&spanned)
}

fn contains_self(ty: &Type, path: &TypePath) -> bool {
    match ty {
        // end recursion
        Type::Path(p) => {
            p == path
                || (p.qself.is_none() && p.path.is_ident("Self"))
                || path_args_contains_self(&p.path, path)
        }
        // recurse to inner type
        Type::Array(arr) => contains_self(&arr.elem, path),
        Type::Group(g) => contains_self(&g.elem, path),
        Type::Paren(t) => contains_self(&t.elem, path),
        Type::Ptr(p) => contains_self(&p.elem, path),
        Type::Reference(p) => contains_self(&p.elem, path),
        Type::Slice(s) => contains_self(&s.elem, path),
        // check deeper
        Type::BareFn(barefn) => {
            return_contains_self(&barefn.output, path)
                || barefn.inputs.iter().any(|i| contains_self(&i.ty, path))
        }
        Type::ImplTrait(it) => bounds_contains_self(it.bounds.iter(), path),
        Type::TraitObject(t) => bounds_contains_self(t.bounds.iter(), path),
        Type::Tuple(t) => t.elems.iter().any(|t| contains_self(t, path)),
        Type::Infer(_) | Type::Macro(_) | Type::Never(_) => false,
        other => {
            let other = other.to_token_stream().to_string();
            other.contains("Self") || other.contains(&path.to_token_stream().to_string())
        }
    }
}

fn ang_generic_contains_self(args: &syn::AngleBracketedGenericArguments, path: &TypePath) -> bool {
    args.args.iter().any(|a| match a {
        syn::GenericArgument::Lifetime(_)
        | syn::GenericArgument::Const(_)
        | syn::GenericArgument::AssocConst(_)
        | syn::GenericArgument::Constraint(_) => false,
        syn::GenericArgument::Type(ty) => contains_self(ty, path),
        syn::GenericArgument::AssocType(assoc) => {
            if contains_self(&assoc.ty, path) {
                return true;
            }
            // TODO: use let-else once we bump MSRV to 1.65.0
            let args = match &assoc.generics {
                Some(args) => args,
                None => return false,
            };
            ang_generic_contains_self(args, path)
        }
        other => {
            let other = other.to_token_stream().to_string();
            other.contains("Self") || other.contains(&path.to_token_stream().to_string())
        }
    })
}

fn return_contains_self(ret: &syn::ReturnType, path: &TypePath) -> bool {
    match &ret {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => contains_self(ty, path),
    }
}

fn bounds_contains_self<'b>(
    mut bounds: impl Iterator<Item = &'b syn::TypeParamBound>,
    path: &TypePath,
) -> bool {
    bounds.any(|b| match b {
        syn::TypeParamBound::Trait(t) => path_args_contains_self(&t.path, path),
        syn::TypeParamBound::Lifetime(_) => false,
        syn::TypeParamBound::Verbatim(ts) => {
            let ts = ts.to_token_stream().to_string();
            ts.contains("Self") || ts.contains(&path.to_token_stream().to_string())
        }
        other => {
            let other = other.to_token_stream().to_string();
            other.contains("Self") || other.contains(&path.to_token_stream().to_string())
        }
    })
}

fn path_args_contains_self(path: &syn::Path, self_path: &syn::TypePath) -> bool {
    match &path.segments.last().unwrap().arguments {
        PathArguments::None => false,
        PathArguments::AngleBracketed(args) => ang_generic_contains_self(args, self_path),
        PathArguments::Parenthesized(args) => {
            return_contains_self(&args.output, self_path)
                || args.inputs.iter().any(|i| contains_self(i, self_path))
        }
    }
}

fn generic_type_idents(generics: Option<Generics>) -> Vec<Ident> {
    generics
        .map(|g| g.type_params().map(|tp| tp.ident.clone()).collect())
        .unwrap_or_default()
}

fn turbofish(idents: &[Ident]) -> TokenStream {
    if idents.is_empty() {
        quote! {}
    } else {
        quote! { :: < #(#idents),* > }
    }
}
