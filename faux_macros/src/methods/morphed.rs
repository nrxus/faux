use crate::self_type::SelfType;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

struct SpanMarker(proc_macro2::Span);

impl Spanned for SpanMarker {
    fn span(&self) -> proc_macro2::Span {
        self.0.clone()
    }
}

pub struct Signature<'a> {
    name: &'a syn::Ident,
    arg_idents: Vec<syn::Ident>,
    is_async: bool,
    output: Option<&'a syn::Type>,
    method_data: Option<MethodData<'a>>,
}

pub struct MethodData<'a> {
    receiver: SelfType,
    receiver_span: SpanMarker,
    name_string: String,
    arg_types: Vec<&'a syn::Type>,
    is_private: bool,
}

impl<'a> Signature<'a> {
    pub fn morph(signature: &'a mut syn::Signature, vis: &syn::Visibility) -> Signature<'a> {
        let is_async = signature.asyncness.is_some();
        let name = &signature.ident;
        let receiver = signature.inputs.first().and_then(|arg| match arg {
            syn::FnArg::Typed(arg) => match &*arg.pat {
                syn::Pat::Ident(pat_ident) if pat_ident.ident == "self" => {
                    Some((SelfType::from_type(&*arg.ty), SpanMarker(arg.ty.span())))
                }
                _ => None,
            },
            syn::FnArg::Receiver(_) => Some((SelfType::Owned, SpanMarker(arg.span()))),
        });

        let len = signature.inputs.len();
        let output = match &signature.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.as_ref()),
        };

        let mut method_data = receiver.map(|(receiver, receiver_span)| MethodData {
            receiver,
            receiver_span,
            name_string: format!("{}", name),
            arg_types: Vec::with_capacity(len - 1),
            is_private: *vis == syn::Visibility::Inherited,
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
                    m.arg_types.push(&*arg.ty);
                }
            });

        Signature {
            is_async,
            name,
            arg_idents,
            output,
            method_data,
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

        let mut block = match &self.method_data {
            // not mockable
            // proxy to real associated function
            None => {
                let mut proxy_real = quote! { <#real_ty>::#name(#(#arg_idents),*) };
                if self.is_async {
                    proxy_real = quote! { #proxy_real.await }
                }
                proxy_real
            }
            // else we can either proxy for real instances
            // or call the mock store for faux instances
            Some(method_data) => {
                let mut proxy_real = quote! { r.#name(#(#arg_idents),*) };
                if self.is_async {
                    proxy_real = quote! { #proxy_real.await }
                }
                let name_string = &method_data.name_string;
                let call_mock = if method_data.is_private {
                    quote! { panic!("faux error: private methods are not mockable; and therefore not directly callable in a mock") }
                } else {
                    let error_msg =
                        format!("'{}::{}' is not mocked", morphed_ty.to_token_stream(), name);
                    quote! {
                        let mut q = q.try_lock().unwrap();
                        unsafe {
                            q.get_mock(#name_string).expect(#error_msg).call((#(#arg_idents),*))
                        }
                    }
                };
                match (&method_data.receiver, real_self) {
                    (SelfType::Owned, _) => quote! {
                        match self {
                            Self(faux::MaybeFaux::Real(r)) => { #proxy_real },
                            Self(faux::MaybeFaux::Faux(q)) => { #call_mock },
                        }
                    },
                    (SelfType::Box, SelfType::Owned) => quote! {
                        match *self {
                            Self(faux::MaybeFaux::Real(r)) => {
                                let r = Box::new(r);
                                #proxy_real
                            },
                            Self(faux::MaybeFaux::Faux(q)) => { #call_mock }
                        }
                    },
                    (SelfType::Box, SelfType::Box) => quote! {
                        match *self {
                            Self(faux::MaybeFaux::Real(r)) => { #proxy_real },
                            Self(faux::MaybeFaux::Faux(ref q)) => { #call_mock },
                        }
                    },
                    (SelfType::Rc, SelfType::Rc) | (SelfType::Arc, SelfType::Arc) => quote! {
                        match *self {
                            Self(faux::MaybeFaux::Real(ref r)) => {
                                let r = r.clone();
                                #proxy_real
                            },
                            Self(faux::MaybeFaux::Faux(ref q)) => { #call_mock }
                        }
                    },
                    (SelfType::Rc, SelfType::Owned) => quote! {
                    let owned = match std::rc::Rc::try_unwrap(self) {
                        Ok(owned) => owned,
                        Err(_) => panic!("faux tried to get a unique instance of Self from Rc<Self> and failed. Consider adding a `self_type = \"Rc\"` to both the #[create] and #[method] attributes tagging this struct and its impl."),
                    };
                    match owned {
                        Self(faux::MaybeFaux::Real(r)) => {
                        let r = std::rc::Rc::new(r);
                        #proxy_real
                        },
                                    Self(faux::MaybeFaux::Faux(q)) => { #call_mock },
                    }
                            },
                    (SelfType::Arc, SelfType::Owned) => quote! {
                    let owned = match std::sync::Arc::try_unwrap(self) {
                                    Ok(owned) => owned,
                                    Err(_) => panic!("faux tried to get a unique instance of Self from Arc<Self> and failed. Consider adding a `self_type = \"Arc\"` to both the #[create] and #[method] attributes tagging this struct and its impl."),
                    };
                    match owned {
                                    Self(faux::MaybeFaux::Real(r)) => {
                        let r = std::sync::Arc::new(r);
                        #proxy_real
                                    },
                                    Self(faux::MaybeFaux::Faux(q)) => { #call_mock },
                    }
                            },
                    (receiver, specified) => {
                        return Err(darling::Error::custom(format!("faux cannot convert from the receiver_type of this method: `{}`, into the self_type specified: `{}`", receiver, specified)).with_span(&method_data.receiver_span));
                    }
                }
            }
        };

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

        let wrapped_self = if let Some(syn::Type::Path(output)) = self.output {
            let last_segment = &output.path.segments.last().unwrap();
            match SelfType::from_segment(last_segment) {
                SelfType::Owned if is_self(output) => Some(match real_self {
                    SelfType::Owned => quote! {{ Self(faux::MaybeFaux::Real(#block))}},
                    generic => {
                        let new_path = generic
                            .new_path()
                            .expect("Generic self should have new() function");
                        quote! {{ Self(faux::MaybeFaux::Real(#new_path(#block)))}}
                    }
                }),
                generic if self_generic(&last_segment.arguments) => {
                    if real_self == generic {
                        let new_path = generic.new_path().expect("return type should not be Self");
                        Some(quote! {{ #new_path(Self(faux::MaybeFaux::Real(#block)))}})
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
        };

        if let Some(wrapped_self) = wrapped_self {
            block = wrapped_self;
        }

        Ok(syn::parse2(quote! {{ #block }}).unwrap())
    }

    pub fn create_when(&self) -> Option<syn::ImplItemMethod> {
        self.method_data
            .as_ref()
            .filter(|m| !m.is_private)
            .map(|m| m.create_when(self.output))
    }
}

impl<'a> MethodData<'a> {
    pub fn create_when(&self, output: Option<&syn::Type>) -> syn::ImplItemMethod {
        let &MethodData {
            ref name_string,
            ref arg_types,
            ..
        } = self;

        let mock_ident = syn::Ident::new(
            &format!("_when_{}", name_string),
            proc_macro2::Span::call_site(),
        );
        let empty = syn::parse2(quote! { () }).unwrap();
        let output = output.unwrap_or(&empty);
        syn::parse2(quote! {
            pub fn #mock_ident(&mut self) -> faux::When<(#(#arg_types),*), #output> {
                match &mut self.0 {
                    faux::MaybeFaux::Faux(faux) => faux::When::new(
                        #name_string,
                        faux.get_mut().unwrap()
                    ),
                    faux::MaybeFaux::Real(_) => panic!("not allowed to mock a real instance!"),
                }
            }
        })
        .unwrap()
    }
}

fn wrong_self_type_error(expected: SelfType, received: SelfType) -> impl std::fmt::Display {
    format!(
        "faux cannot create {expected}<Self> from a self type of {received}. Consider specifying a different self_type in the faux attributes, or moving this method to a non-faux impl block",
        expected = expected,
        received = received
    )
}
