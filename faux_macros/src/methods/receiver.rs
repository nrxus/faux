use crate::self_type::SelfType;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use std::{
    boxed::Box,
    fmt::{self, Formatter},
};

#[derive(Debug)]
pub struct Receiver {
    pub kind: SelfKind,
    pub ty: Box<syn::Type>,
}

impl Receiver {
    pub fn from_signature(signature: &syn::Signature) -> darling::Result<Option<Self>> {
        let receiver = match signature.inputs.first() {
            Some(syn::FnArg::Receiver(receiver)) => receiver,
            _ => return Ok(None),
        };

        let kind = if receiver.colon_token.is_some() {
            SelfKind::from_type(receiver.ty.as_ref())?
        } else {
            match (&receiver.reference, &receiver.mutability) {
                (None, _) => SelfKind::Owned,
                (Some(_), None) => SelfKind::Pointer(PointerKind::Ref),
                (Some(_), Some(_)) => SelfKind::Pointer(PointerKind::MutRef),
            }
        };

        Ok(Some(Receiver {
            kind,
            ty: receiver.ty.clone(),
        }))
    }

    pub fn method_body(
        &self,
        self_type: SelfType,
        proxy_real: Option<TokenStream>,
        call_stub: TokenStream,
    ) -> darling::Result<syn::Expr> {
        let get_self = match &self.kind {
            SelfKind::Owned
            | SelfKind::Pointer(PointerKind::Ref)
            | SelfKind::Pointer(PointerKind::MutRef) => {
                quote! { self }
            }
            SelfKind::Pointer(PointerKind::Arc) | SelfKind::Pointer(PointerKind::Rc) => {
                quote! { &*self }
            }
            SelfKind::Pointer(PointerKind::Box) => quote! { *self },
            SelfKind::Pointer(PointerKind::Pin(p)) => {
                let unpinned = quote! { unsafe { std::pin::Pin::into_inner_unchecked(self) } };
                match **p {
                    PointerKind::Ref | PointerKind::MutRef => unpinned,
                    PointerKind::Rc | PointerKind::Arc => {
                        let panic_msg = "faux tried to get a unique instance of Self and failed";
                        let self_of_receiver = match **p {
                            PointerKind::Arc => SelfType::Arc,
                            PointerKind::Rc => SelfType::Rc,
                            _ => unreachable!(),
                        };
                        let path = self_of_receiver.path().unwrap();
                        quote! {{
                            match #path::try_unwrap(#unpinned) {
                                Ok(owned) => owned,
                                Err(_) => panic!(#panic_msg),
                            }
                        }}
                    }
                    PointerKind::Box => quote! { *#unpinned },
                    PointerKind::Pin(_) => {
                        return Err(darling::Error::custom("faux does not support nest Pins")
                            .with_span(self));
                    }
                }
            }
        };

        let kind = &self.kind;

        let proxy_real = proxy_real.map(|proxy_real| {
            let proxy_real = match (kind, self_type) {
                (SelfKind::Owned, SelfType::Owned)
                    | (SelfKind::Pointer(PointerKind::Ref), SelfType::Owned)
                    | (SelfKind::Pointer(PointerKind::MutRef), SelfType::Owned)
                    | (SelfKind::Pointer(PointerKind::Box), SelfType::Box) => proxy_real,
                (SelfKind::Pointer(PointerKind::Ref), _) => quote! {
                    let _maybe_faux_real = &*_maybe_faux_real;
                    #proxy_real
                },
                (SelfKind::Pointer(PointerKind::MutRef), SelfType::Box) => quote! {
                    let _maybe_faux_real = &mut *_maybe_faux_real;
                    #proxy_real
                },
                (SelfKind::Owned, SelfType::Box) => quote! {
                    let _maybe_faux_real = *_maybe_faux_real;
                    #proxy_real
                },
                (SelfKind::Pointer(PointerKind::Box), SelfType::Owned) => quote! {
                    let _maybe_faux_real = std::boxed::Box::new(_maybe_faux_real);
                    #proxy_real
                },
                (SelfKind::Pointer(PointerKind::Rc), SelfType::Rc)
                    | (SelfKind::Pointer(PointerKind::Arc), SelfType::Arc) => {
                        quote! {
                            let _maybe_faux_real = _maybe_faux_real.clone();
                            #proxy_real
                        }
                    }
                (SelfKind::Pointer(PointerKind::Rc), SelfType::Owned)
                    | (SelfKind::Pointer(PointerKind::Arc), SelfType::Owned) => {
                        let self_of_receiver = match kind {
                            SelfKind::Pointer(PointerKind::Arc) => SelfType::Arc,
                            SelfKind::Pointer(PointerKind::Rc) => SelfType::Rc,
                            _ => unreachable!(),
                        };
                        let path = self_of_receiver.path();
                        let new_path = self_of_receiver.new_path().unwrap();
                        let panic_msg = format!("faux tried to get a unique instance of Self from  and failed. Consider adding a `self_type = \"{}\"` argument to both the #[create] and #[method] attributes tagging this struct and its impl.", self_of_receiver);

                        quote! {
                            let owned = match #path::try_unwrap(self) {
                                Ok(owned) => owned,
                                Err(_) => panic!(#panic_msg),
                            };

                            if let Self(faux::MaybeFaux::Real(_maybe_faux_real)) = owned {
                                let _maybe_faux_real = #new_path(_maybe_faux_real);
                                #proxy_real
                            } else {
                                unreachable!()
                            }
                        }
                    }
                (SelfKind::Pointer(PointerKind::Pin(pointer)), SelfType::Owned) => match **pointer {
                    PointerKind::Ref | PointerKind::MutRef => quote! {
                        let _maybe_faux_real = unsafe { std::pin::Pin::new_unchecked(_maybe_faux_real) };
                        #proxy_real
                    },
                    PointerKind::Box => quote! {
                        let _maybe_faux_real = unsafe { std::pin::Pin::new_unchecked(std::boxed::Box::new(_maybe_faux_real)) };
                        #proxy_real
                    },
                    PointerKind::Rc | PointerKind::Arc => {
                        let self_of_receiver = match **pointer {
                            PointerKind::Arc => SelfType::Arc,
                            PointerKind::Rc => SelfType::Rc,
                            _ => unreachable!(),
                        };
                        let new_path = self_of_receiver.new_path().unwrap();

                        quote! {
                            let _maybe_faux_real = unsafe { std::pin::Pin::new_unchecked(#new_path(_maybe_faux_real)) };
                            #proxy_real
                        }
                    }
                    PointerKind::Pin(_) => unreachable!(),
                },
                (receiver, specified) => {
                    return Err(darling::Error::custom(format!("faux cannot convert from the receiver_type of this method: `{}`, into the self_type specified: `{}`", receiver, specified)).with_span(self));
                }
            };
            Ok(proxy_real)
        }).transpose()?;

        let get_self: syn::Expr = syn::parse2(get_self).expect("failed to parse get_self");
        let body = match proxy_real {
            Some(proxy_real) => syn::parse_quote! {
                match #get_self {
                    Self(faux::MaybeFaux::Real(_maybe_faux_real)) => { #proxy_real },
                    Self(faux::MaybeFaux::Faux(_maybe_faux_faux)) => { #call_stub },
                }
            },
            None => {
                syn::parse_quote! {
                    match #get_self {
                        Self(_maybe_faux_faux) => { #call_stub },
                    }
                }
            }
        };

        Ok(body)
    }
}

#[derive(Clone, Debug)]
pub enum SelfKind {
    Owned,
    Pointer(PointerKind),
}

#[derive(Clone, Debug)]
pub enum PointerKind {
    Ref,
    MutRef,
    Rc,
    Arc,
    Box,
    Pin(Box<PointerKind>),
}

impl SelfKind {
    pub fn from_type(ty: &syn::Type) -> darling::Result<Self> {
        if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
            if path.is_ident("Self") {
                return Ok(SelfKind::Owned);
            }
        }

        PointerKind::from_type(ty).map(SelfKind::Pointer)
    }
}

impl fmt::Display for SelfKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SelfKind::Owned => write!(f, "Self"),
            SelfKind::Pointer(p) => write!(f, "{}", p),
        }
    }
}

impl fmt::Display for PointerKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PointerKind::Ref => write!(f, "&Self"),
            PointerKind::MutRef => write!(f, "&mut Self"),
            PointerKind::Rc => write!(f, "Rc<Self"),
            PointerKind::Arc => write!(f, "Arc<Self>"),
            PointerKind::Box => write!(f, "Box<Self>"),
            PointerKind::Pin(p) => write!(f, "Pin<{}>", p),
        }
    }
}

impl ToTokens for Receiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ty.to_tokens(tokens)
    }
}

impl PointerKind {
    pub fn from_type(ty: &syn::Type) -> darling::Result<Self> {
        match ty {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                let ty = path.segments.last().unwrap();
                match &ty.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        if args.args.len() != 1 {
                            return Err(darling::Error::custom(
                                "faux does not support this kind of self type",
                            )
                            .with_span(ty));
                        }
                        let arg = match args.args.last().unwrap() {
                            syn::GenericArgument::Type(gen_ty) => gen_ty,
                            _ => {
                                return Err(darling::Error::custom(
                                    "faux does not support this kind of self type",
                                )
                                .with_span(ty))
                            }
                        };

                        let ident = &ty.ident;
                        // can't match on Ident
                        if ident == "Rc" {
                            Ok(PointerKind::Rc)
                        } else if ident == "Arc" {
                            Ok(PointerKind::Arc)
                        } else if ident == "Box" {
                            Ok(PointerKind::Box)
                        } else if ident == "Pin" {
                            let pointer = PointerKind::from_type(arg)?;
                            Ok(PointerKind::Pin(Box::new(pointer)))
                        } else {
                            Err(darling::Error::custom(
                                "faux does not support this kind of pointer type",
                            )
                            .with_span(ty))
                        }
                    }
                    _ => Err(darling::Error::custom(
                        "faux does not support this kind of path arguments in a self type",
                    )
                    .with_span(ty)),
                }
            }
            syn::Type::Reference(reference) => match reference.mutability {
                None => Ok(PointerKind::Ref),
                Some(_) => Ok(PointerKind::MutRef),
            },
            _ => Err(
                darling::Error::custom("faux does not support this kind of self type")
                    .with_span(ty),
            ),
        }
    }
}
