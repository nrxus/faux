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

impl SelfType {
    pub fn from_type(ty: &syn::Type) -> Self {
        match ty {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                Self::from_segment(&path.segments.last().unwrap())
            }
            _ => SelfType::Owned,
        }
    }

    pub fn from_segment(segment: &syn::PathSegment) -> Self {
        let ident = &segment.ident;

        // can't match on Ident
        if ident == "Rc" {
            SelfType::Rc
        } else if ident == "Arc" {
            SelfType::Arc
        } else if ident == "Box" {
            SelfType::Box
        } else {
            SelfType::Owned
        }
    }

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
            SelfType::Owned => "owned",
            SelfType::Rc => "Rc",
            SelfType::Arc => "Arc",
            SelfType::Box => "Box",
        })
        .fmt(f)
    }
}
