use crate::self_type::SelfType;
use quote::{quote, ToTokens};

pub struct Signature<'a> {
    name: &'a syn::Ident,
    arg_idents: Vec<syn::Ident>,
    is_async: bool,
    output: Option<&'a syn::Type>,
    mockable_data: Option<MockableData<'a>>,
}

pub struct MockableData<'a> {
    receiver: SelfType,
    name_string: String,
    arg_types: Vec<&'a syn::Type>,
}

impl<'a> Signature<'a> {
    pub fn morph(signature: &'a mut syn::Signature) -> Signature<'a> {
        let is_async = signature.asyncness.is_some();
        let name = &signature.ident;
        let receiver = match signature.inputs.first() {
            None => None,
            Some(syn::FnArg::Receiver(_)) => Some(SelfType::Owned),
            Some(syn::FnArg::Typed(arg)) => match &*arg.pat {
                syn::Pat::Ident(pat_ident) if pat_ident.ident == "self" => {
                    Some(SelfType::from_type(&*arg.ty))
                }
                _ => None,
            },
        };

        let len = signature.inputs.len();
        let output = match &signature.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.as_ref()),
        };

        let mut mockable_data = receiver.map(|receiver| MockableData {
            receiver,
            name_string: format!("{}", name),
            arg_types: Vec::with_capacity(len - 1),
        });

        let mut arg_idents =
            Vec::with_capacity(signature.inputs.len() - mockable_data.is_some() as usize);

        signature
            .inputs
            .iter_mut()
            .skip(mockable_data.is_some() as usize) // if it's a method; skip first arg
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
                if let Some(m) = &mut mockable_data {
                    m.arg_types.push(&*arg.ty);
                }
            });

        Signature {
            is_async,
            name,
            arg_idents,
            output,
            mockable_data,
        }
    }

    pub fn create_body(
        &self,
        real_self: &SelfType,
        real_ty: &syn::TypePath,
        morphed_ty: &syn::TypePath,
    ) -> syn::Block {
        let name = self.name;
        let arg_idents = &self.arg_idents;

        let mut block = match &self.mockable_data {
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
            Some(mockable_data) => {
                let mut proxy_real = quote! { r.#name(#(#arg_idents),*) };
                if self.is_async {
                    proxy_real = quote! { #proxy_real.await }
                }
                let name_string = &mockable_data.name_string;
                let call_mock = {
                    let error_msg =
                        format!("'{}::{}' is not mocked", morphed_ty.to_token_stream(), name);
                    quote! {
                        let mut q = q.try_lock().unwrap();
                        unsafe {
                            q.get_mock(#name_string).expect(#error_msg).call((#(#arg_idents),*))
                        }
                    }
                };
                match (&mockable_data.receiver, real_self) {
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
                    (SelfType::Rc, SelfType::Rc) | (SelfType::Arc, SelfType::Arc) => quote! {
                        match *self {
                            Self(faux::MaybeFaux::Real(ref r)) => {
                                let r = r.clone();
                                #proxy_real
                            },
                            Self(faux::MaybeFaux::Faux(ref q)) => { #call_mock }
                        }
                    },
                    (SelfType::Box, SelfType::Box) => quote! {
                        match *self {
                            Self(faux::MaybeFaux::Real(r)) => { #proxy_real },
                            Self(faux::MaybeFaux::Faux(ref q)) => { #call_mock },
                        }
                    },
                    _ => todo!(),
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

        let wrapped_self = self.output
            .and_then(|output| match output {
                syn::Type::Path(ty) => Some(ty),
                _ => None,
            })
            .and_then(|output| {
                let last_segment = &output.path.segments.last().unwrap();
                match SelfType::from_segment(last_segment) {
                    SelfType::Owned if is_self(output) => Some(match real_self {
                        SelfType::Owned => quote! {{ Self(faux::MaybeFaux::Real(#block))}},
                        SelfType::Rc => quote! {{ Self(faux::MaybeFaux::Real(std::rc::Rc::new(#block)))}},
                        SelfType::Arc => quote! {{ Self(faux::MaybeFaux::Real(std::sync::Arc::new(#block)))}},
                        SelfType::Box => quote! {{ Self(faux::MaybeFaux::Real(std::boxed::Box::new(#block)))}},
                    }),
                    SelfType::Rc if self_generic(&last_segment.arguments) => {
                        Some(match real_self {
                            SelfType::Rc => {
                                quote! {{ std::rc::Rc::new(Self(faux::MaybeFaux::Real(#block)))}}
                            }
                            _ => panic!("faux cannot create an Rc<Self> from a different SelfType. Consider specifying a different an Rc self_tupe in the faux attributes, or moving this method to a non-faux impl block")
                        })
                    }
                    SelfType::Arc if self_generic(&last_segment.arguments) => {
                        Some(match real_self {
                            SelfType::Arc => {
                                quote! {{ std::sync::Arc::new(Self(faux::MaybeFaux::Real(#block)))}}
                            }
                            _ => panic!("faux cannot create an Arc<Self> from a different SelfType. Consider specifying a different an Arc self_tupe in the faux attributes, or moving this method to a non-faux impl block")
                        })
                    }
                    SelfType::Box if self_generic(&last_segment.arguments) => {
                        Some(match real_self {
                            SelfType::Box => {
                                quote! {{ std::boxed::Box::new(Self(faux::MaybeFaux::Real(#block)))}}
                            }
                            _ => panic!("faux cannot create a Box<Self> from a different SelfType. Consider specifying a different a Box self_tupe in the faux attributes, or moving this method to a non-faux impl block")
                        })
                    }
                    _ => None,
                }
            });

        if let Some(wrapped_self) = wrapped_self {
            block = wrapped_self;
        }

        syn::parse2(quote! {{ #block }}).unwrap()
    }

    pub fn create_when(&self) -> Option<syn::ImplItemMethod> {
        self.mockable_data
            .as_ref()
            .map(|m| m.create_when(self.output))
    }
}

impl<'a> MockableData<'a> {
    pub fn create_when(&self, output: Option<&syn::Type>) -> syn::ImplItemMethod {
        let &MockableData {
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
