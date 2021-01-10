use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

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
    /// Get the path to create a new instance of the given self-type, if one is known
    /// from the standard library.
    pub fn new_path(self) -> Option<TokenStream> {
        self.path().map(|p| quote! { #p::new })
    }

    pub fn path(self) -> Option<TokenStream> {
        match self {
            SelfType::Owned => None,
            SelfType::Rc => Some(quote!(std::rc::Rc)),
            SelfType::Box => Some(quote!(std::boxed::Box)),
            SelfType::Arc => Some(quote!(std::sync::Arc)),
        }
    }

    pub fn from_path(type_path: &syn::TypePath) -> Self {
        let segment = type_path.path.segments.last().unwrap();
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
}

impl Default for SelfType {
    fn default() -> Self {
        SelfType::Owned
    }
}

impl std::fmt::Display for SelfType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (match self {
            SelfType::Owned => "owned",
            SelfType::Rc => "Rc",
            SelfType::Arc => "Arc",
            SelfType::Box => "Box",
        })
        .fmt(f)
    }
}
