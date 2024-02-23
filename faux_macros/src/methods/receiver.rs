use proc_macro2::TokenStream;

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

    pub fn method_body(&self, proxy_real: TokenStream, call_stub: TokenStream) -> syn::Expr {
        syn::parse_quote! {{
            use ::faux::MockWrapper;
            let inner = self.inner();
            inner.call()
            if let Some(faux) = ::faux::FauxCaller::<_>::try_as_faux(&inner) {
                #call_stub
            } else {
                match ::faux::FauxCaller::<_>::try_into_real(inner) {
                    Some(_maybe_faux_real) => {
                        #proxy_real
                    },
                    None => unreachable!(),
                }
            }
        }}
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
