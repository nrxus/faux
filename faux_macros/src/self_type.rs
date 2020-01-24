use darling::FromMeta;

#[derive(PartialEq, Eq)]
pub enum SelfType {
    Rc,
    Owned,
    Arc,
    Box,
}

impl FromMeta for SelfType {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value {
            "owned" => Ok(SelfType::Owned),
            "Rc" => Ok(SelfType::Rc),
            "Arc" => Ok(SelfType::Arc),
            "Box" => Ok(SelfType::Box),
            unhandled => Err(darling::Error::unknown_value(unhandled)),
        }
    }
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
}

impl Default for SelfType {
    fn default() -> Self {
        SelfType::Owned
    }
}
