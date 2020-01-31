use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use std::fmt;

#[derive(FromMeta, PartialEq, Eq, Copy, Clone)]
#[darling(rename_all = "PascalCase")]
pub enum SelfType {
    Rc,
    #[darling(rename = "owned")]
    Owned,
    Arc,
    Box,
}

impl Default for OwnedType {
    fn default() -> Self {
        OwnedType::Value
    }
}

impl SelfType {
    /// Get the path to create a new instance of the given self-type, if one is known
    /// from the standard library.
    pub fn new_path(self) -> Option<TokenStream> {
        match self {
            SelfType::Owned => None,
            SelfType::Rc => Some(quote!(std::rc::Rc::new)),
            SelfType::Box => Some(quote!(std::boxed::Box::new)),
            SelfType::Arc => Some(quote!(std::sync::Arc::new)),
        }
    }
}

impl Default for SelfType {
    fn default() -> Self {
        SelfType::Owned
    }
}

impl fmt::Display for SelfType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (match self {
            SelfType::Owned => "Self",
            SelfType::Rc => "Rc<Self>",
            SelfType::Arc => "Arc<Self>",
            SelfType::Box => "Box<Self>",
        })
        .fmt(f)
    }
}

pub enum ReceiverType {
    Rc,
    Arc,
    Box,
    Owned(OwnedType),
}

impl ReceiverType {
    pub fn matches(&self, self_type: &SelfType) -> bool {
        match (self, self_type) {
            (ReceiverType::Rc, SelfType::Rc) => true,
            (ReceiverType::Arc, SelfType::Arc) => true,
            (ReceiverType::Box, SelfType::Box) => true,
            (ReceiverType::Owned(_), SelfType::Owned) => true,
            _ => true,
        }
    }

    pub fn from_type(ty: &syn::Type) -> Self {
        match ty {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                Self::from_segment(&path.segments.last().unwrap())
            }
            syn::Type::Reference(reference) => match reference.mutability {
                None => ReceiverType::Owned(OwnedType::Ref),
                Some(_) => ReceiverType::Owned(OwnedType::MutRef),
            },
            _ => ReceiverType::Owned(OwnedType::Value),
        }
    }

    pub fn from_segment(segment: &syn::PathSegment) -> Self {
        let ident = &segment.ident;

        // can't match on Ident
        if ident == "Rc" {
            ReceiverType::Rc
        } else if ident == "Arc" {
            ReceiverType::Arc
        } else if ident == "Box" {
            ReceiverType::Box
        } else {
            ReceiverType::Owned(OwnedType::Value)
        }
    }
}

impl quote::ToTokens for OwnedType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            OwnedType::Value => {}
            OwnedType::Ref => tokens.extend(quote! { & }),
            OwnedType::MutRef => tokens.extend(quote! { &mut }),
        }
    }
}

impl fmt::Display for ReceiverType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (match self {
            ReceiverType::Owned(OwnedType::Value) => "Self",
            ReceiverType::Owned(OwnedType::Ref) => "&Self",
            ReceiverType::Owned(OwnedType::MutRef) => "&mut Self",
            ReceiverType::Rc => "Rc<Self>",
            ReceiverType::Arc => "Arc<Self>",
            ReceiverType::Box => "Box<Self>",
        })
        .fmt(f)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum OwnedType {
    Value,
    Ref,
    MutRef,
}
