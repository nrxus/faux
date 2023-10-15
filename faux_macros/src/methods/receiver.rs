use crate::self_type::SelfType;

use proc_macro2::TokenStream;
use quote::quote;

use std::{
    boxed::Box,
    fmt::{self, Formatter},
};

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
    pub fn new(receiver: &syn::Receiver) -> Self {
        if receiver.colon_token.is_some() {
            if let syn::Type::Path(syn::TypePath { path, .. }) = receiver.ty.as_ref() {
                if path.is_ident("Self") {
                    return SelfKind::Owned;
                }
            }

            let pointer = PointerKind::from_type(receiver.ty.as_ref()).unwrap();
            SelfKind::Pointer(pointer)
        } else {
            match (&receiver.reference, &receiver.mutability) {
                (None, _) => SelfKind::Owned,
                (Some(_), None) => SelfKind::Pointer(PointerKind::Ref),
                (Some(_), Some(_)) => SelfKind::Pointer(PointerKind::MutRef),
            }
        }
    }

    fn matchable_self(&self) -> darling::Result<TokenStream> {
        let get_self = match self {
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
                        return Err(darling::Error::custom("faux does not support nest Pins"));
                    }
                }
            }
        };

        Ok(get_self)
    }

    pub fn method_body(
        &self,
        self_type: SelfType,
        proxy_real: TokenStream,
        call_stub: TokenStream,
    ) -> darling::Result<syn::Expr> {
        let get_self = self.matchable_self()?;

        let project_real = match self.project_real(self_type) {
            Ok(value) => value,
            Err(value) => return value,
        };

        let proxy_real = quote! {
            #project_real
            #proxy_real
        };

        Ok(syn::parse_quote! {
            match &self.0 {
                faux::MaybeFaux::Faux(_) => { #call_stub },
                _ => match #get_self {
                    Self(faux::MaybeFaux::Real(_maybe_faux_real)) => { #proxy_real },
                    Self(faux::MaybeFaux::Faux(_)) => unreachable!(),
                }
            }
        })
    }

    fn project_real(&self, self_type: SelfType) -> Result<TokenStream, darling::Result<syn::Expr>> {
        Ok(match (self, self_type) {
            (SelfKind::Owned, SelfType::Owned)
            | (SelfKind::Pointer(PointerKind::Ref), SelfType::Owned)
            | (SelfKind::Pointer(PointerKind::MutRef), SelfType::Owned)
            | (SelfKind::Pointer(PointerKind::Box), SelfType::Box) => quote! {},
            (SelfKind::Pointer(PointerKind::Ref), _) => quote! {
                let _maybe_faux_real = &*_maybe_faux_real;
            },
            (SelfKind::Pointer(PointerKind::MutRef), SelfType::Box) => quote! {
                let _maybe_faux_real = &mut *_maybe_faux_real;
            },
            (SelfKind::Owned, SelfType::Box) => quote! {
                let _maybe_faux_real = *_maybe_faux_real;
            },
            (SelfKind::Pointer(PointerKind::Box), SelfType::Owned) => quote! {
                let _maybe_faux_real = std::boxed::Box::new(_maybe_faux_real);
            },
            (SelfKind::Pointer(PointerKind::Rc), SelfType::Rc)
            | (SelfKind::Pointer(PointerKind::Arc), SelfType::Arc) => quote! {
                    let _maybe_faux_real = _maybe_faux_real.clone();
            },
            (SelfKind::Pointer(PointerKind::Rc), SelfType::Owned)
            | (SelfKind::Pointer(PointerKind::Arc), SelfType::Owned) => {
                let self_of_receiver = match self {
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

                    let _maybe_faux_real = match owned {
                        Self(faux::MaybeFaux::Real(_maybe_faux_real)) => #new_path(_maybe_faux_real),
                        _ => unreachable!()
                    };
                }
            }
            (SelfKind::Pointer(PointerKind::Pin(pointer)), SelfType::Owned) => match **pointer {
                PointerKind::Ref | PointerKind::MutRef => quote! {
                    let _maybe_faux_real = unsafe { std::pin::Pin::new_unchecked(_maybe_faux_real) };
                },
                PointerKind::Box => quote! {
                    let _maybe_faux_real = unsafe { std::pin::Pin::new_unchecked(std::boxed::Box::new(_maybe_faux_real)) };
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
                    }
                }
                PointerKind::Pin(_) => unreachable!(),
            },
            (receiver, specified) => {
                return Err(Err(darling::Error::custom(format!("faux cannot convert from the receiver_type of this method: `{}`, into the self_type specified: `{}`", receiver, specified))));
            }
        })
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
