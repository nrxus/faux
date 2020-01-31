use crate::self_type::SelfType;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

pub struct Receiver {
    span: Span,
    pub kind: Kind,
}

impl Receiver {
    pub fn from_signature(signature: &syn::Signature) -> Option<Self> {
        signature.inputs.first().and_then(|arg| match arg {
            syn::FnArg::Typed(arg) => match &*arg.pat {
                syn::Pat::Ident(pat_ident) if pat_ident.ident == "self" => Some(Receiver {
                    kind: Kind::from_type(&*arg.ty),
                    span: arg.ty.span(),
                }),
                _ => None,
            },
            syn::FnArg::Receiver(receiver) => Some(Receiver {
                kind: match (&receiver.reference, &receiver.mutability) {
                    (None, _) => Kind::Owned(OwnedKind::Value),
                    (Some(_), None) => Kind::Owned(OwnedKind::Ref),
                    (Some(_), Some(_)) => Kind::Owned(OwnedKind::MutRef),
                },
                span: arg.span(),
            }),
        })
    }

    pub fn method_body(
        &self,
        self_type: SelfType,
        proxy_real: TokenStream,
        call_mock: TokenStream,
    ) -> darling::Result<TokenStream> {
        let get_self = match &self.kind {
            Kind::Owned(_) => quote! { self },
            Kind::Arc | Kind::Rc => quote! { &*self },
            Kind::Box => quote! { *self },
        };

        let kind = &self.kind;

        let proxy_real = match (kind, self_type) {
            (Kind::Owned(_), SelfType::Owned) | (Kind::Box, SelfType::Box) => proxy_real,
            (Kind::Owned(OwnedKind::Ref), _) => quote! {
                let r = &*r;
                #proxy_real
            },
            (Kind::Owned(owned_type), SelfType::Box) => quote! {
                let r = #owned_type *r;
                #proxy_real
            },
            (Kind::Box, SelfType::Owned) => quote! {
                let r = std::boxed::Box::new(r);
                #proxy_real
            },
            (Kind::Rc, SelfType::Rc) | (Kind::Arc, SelfType::Arc) => quote! {
                let r = r.clone();
                #proxy_real
            },
            (Kind::Rc, SelfType::Owned) | (Kind::Arc, SelfType::Owned) => {
                let self_of_receiver = kind.to_self_type();
                let path = self_of_receiver.path();
                let new_path = self_of_receiver.new_path().unwrap();
                let panic_msg = format!("faux tried to get a unique instance of Self from  and failed. Consider adding a `self_type = \"{}\"` argument to both the #[create] and #[method] attributes tagging this struct and its impl.", self_of_receiver);

                quote! {
                    let owned = match #path::try_unwrap(self) {
                        Ok(owned) => owned,
                        Err(_) => panic!(#panic_msg),
                    };

                    if let Self(faux::MaybeFaux::Real(r)) = owned {
                        let r = #new_path(r);
                        #proxy_real
                    }
                }
            }
            (receiver, specified) => {
                return Err(darling::Error::custom(format!("faux cannot convert from the receiver_type of this method: `{}`, into the self_type specified: `{}`", receiver, specified)).with_span(self));
            }
        };

        Ok(quote! {
            match #get_self {
                Self(faux::MaybeFaux::Real(r)) => { #proxy_real },
                Self(faux::MaybeFaux::Faux(q)) => { #call_mock },
            }
        })
    }
}

pub enum Kind {
    Rc,
    Arc,
    Box,
    Owned(OwnedKind),
}

impl Kind {
    pub fn matches(&self, self_type: &SelfType) -> bool {
        match (self, self_type) {
            (Kind::Rc, SelfType::Rc) => true,
            (Kind::Arc, SelfType::Arc) => true,
            (Kind::Box, SelfType::Box) => true,
            (Kind::Owned(_), SelfType::Owned) => true,
            _ => true,
        }
    }

    pub fn from_type(ty: &syn::Type) -> Self {
        match ty {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                Self::from_segment(&path.segments.last().unwrap())
            }
            syn::Type::Reference(reference) => match reference.mutability {
                None => Kind::Owned(OwnedKind::Ref),
                Some(_) => Kind::Owned(OwnedKind::MutRef),
            },
            _ => Kind::Owned(OwnedKind::Value),
        }
    }

    pub fn from_segment(segment: &syn::PathSegment) -> Self {
        let ident = &segment.ident;

        // can't match on Ident
        if ident == "Rc" {
            Kind::Rc
        } else if ident == "Arc" {
            Kind::Arc
        } else if ident == "Box" {
            Kind::Box
        } else {
            Kind::Owned(OwnedKind::Value)
        }
    }

    pub fn to_self_type(&self) -> SelfType {
        match self {
            Kind::Arc => SelfType::Arc,
            Kind::Rc => SelfType::Rc,
            Kind::Box => SelfType::Box,
            Kind::Owned(_) => SelfType::Owned,
        }
    }
}

impl quote::ToTokens for OwnedKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            OwnedKind::Value => {}
            OwnedKind::Ref => tokens.extend(quote! { & }),
            OwnedKind::MutRef => tokens.extend(quote! { &mut }),
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (match self {
            Kind::Owned(OwnedKind::Value) => "Self",
            Kind::Owned(OwnedKind::Ref) => "&Self",
            Kind::Owned(OwnedKind::MutRef) => "&mut Self",
            Kind::Rc => "Rc<Self>",
            Kind::Arc => "Arc<Self>",
            Kind::Box => "Box<Self>",
        })
        .fmt(f)
    }
}

impl Spanned for Receiver {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum OwnedKind {
    Value,
    Ref,
    MutRef,
}
