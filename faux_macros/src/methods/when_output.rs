use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

#[derive(Debug)]
pub struct WhenOutput {
    pub ty: Box<syn::Type>,
    pub lifetimes: Vec<syn::Lifetime>,
    pub dynamized: bool,
}

impl ToTokens for WhenOutput {
    fn to_tokens(&self, token_stream: &mut proc_macro2::TokenStream) {
        if self.dynamized {
            let ty = &self.ty;
            token_stream.extend(quote! { std::boxed::Box<#ty> })
        } else {
            self.ty.to_tokens(token_stream)
        }
    }
}

impl WhenOutput {
    pub fn new(ty: syn::ReturnType, implicit_lifetime: Option<&syn::Lifetime>) -> Self {
        match ty {
            syn::ReturnType::Default => WhenOutput {
                ty: syn::parse_quote! { () },
                lifetimes: vec![],
                dynamized: false,
            },
            syn::ReturnType::Type(_, mut ty) => {
                let (lifetimes, dynamized) =
                    WhenOutput::sanitize(&mut ty, implicit_lifetime, &mut 0);
                WhenOutput {
                    ty,
                    lifetimes,
                    dynamized,
                }
            }
        }
    }

    fn sanitize(
        ty: &mut syn::Type,
        implicit_lifetime: Option<&syn::Lifetime>,
        lifetime_id: &mut usize,
    ) -> (Vec<syn::Lifetime>, bool) {
        fn new_lt(lifetime_id: &mut usize, span: Span) -> syn::Lifetime {
            let new_lt = format!("'_faux_out_{}", lifetime_id);
            *lifetime_id += 1;
            syn::Lifetime::new(&new_lt, span)
        }

        match ty {
            syn::Type::Array(syn::TypeArray { elem, .. })
            | syn::Type::Group(syn::TypeGroup { elem, .. })
            | syn::Type::Paren(syn::TypeParen { elem, .. })
            | syn::Type::Ptr(syn::TypePtr { elem, .. })
            | syn::Type::Slice(syn::TypeSlice { elem, .. }) => {
                Self::sanitize(elem, implicit_lifetime, lifetime_id)
            }
            syn::Type::ImplTrait(t) => {
                let mut new_lifetimes = vec![];
                let lifetimes = t.bounds.iter_mut().filter_map(|b| match b {
                    syn::TypeParamBound::Lifetime(lt) => Some(lt),
                    _ => None,
                });

                let mut any_lifetime = false;
                for lt in lifetimes {
                    any_lifetime = true;
                    if lt.ident == "_" {
                        if let Some(implicit_lifetime) = implicit_lifetime {
                            *lt = implicit_lifetime.clone();
                        }
                    }
                }

                if !any_lifetime {
                    let new_lt = new_lt(lifetime_id, t.span());
                    t.bounds.push(syn::TypeParamBound::Lifetime(new_lt.clone()));
                    new_lifetimes.push(new_lt);
                }

                let send = syn::TypeParamBound::Trait(syn::TraitBound {
                    paren_token: None,
                    modifier: syn::TraitBoundModifier::None,
                    lifetimes: None,
                    path: Ident::new("Send", proc_macro2::Span::call_site()).into(),
                });

                if !t.bounds.iter().any(|b| *b == send) {
                    t.bounds.push(send);
                }

                *ty = syn::Type::Paren(syn::TypeParen {
                    paren_token: syn::token::Paren(t.span()),
                    elem: Box::new(syn::Type::TraitObject(syn::TypeTraitObject {
                        bounds: std::mem::take(&mut t.bounds),
                        dyn_token: Some(syn::Token![dyn](proc_macro2::Span::call_site())),
                    })),
                });

                (new_lifetimes, true)
            }
            syn::Type::Path(syn::TypePath { path, .. }) => {
                let last = path.segments.last_mut().unwrap();
                let args = match &mut last.arguments {
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        args,
                        ..
                    }) => args,
                    _ => return (vec![], false),
                };

                let mut any_dynamized = false;
                let mut joined_lifetimes = vec![];
                for arg in args.iter_mut() {
                    match arg {
                        syn::GenericArgument::Lifetime(lt) if lt.ident == "_" => {
                            if let Some(implicit_lifetime) = implicit_lifetime {
                                *lt = implicit_lifetime.clone();
                            }
                        }
                        syn::GenericArgument::Type(ty) => {
                            let (lifetimes, dynamized) =
                                Self::sanitize(ty, implicit_lifetime, lifetime_id);
                            joined_lifetimes.extend(lifetimes);
                            any_dynamized |= dynamized;
                        }
                        _ => {}
                    }
                }

                (joined_lifetimes, any_dynamized)
            }
            syn::Type::Reference(ty) => {
                if matches!(&ty.lifetime, Some(lt) if lt.ident != "_") {
                    Self::sanitize(&mut ty.elem, implicit_lifetime, lifetime_id)
                } else {
                    ty.lifetime = implicit_lifetime.cloned();
                    Self::sanitize(&mut ty.elem, implicit_lifetime, lifetime_id)
                }
            }
            syn::Type::Tuple(t) => {
                let mut any_dynamized = false;
                let lifetimes = t
                    .elems
                    .iter_mut()
                    .flat_map(|ty| {
                        let (lts, dynamized) = Self::sanitize(ty, implicit_lifetime, lifetime_id);
                        any_dynamized |= dynamized;
                        lts
                    })
                    .collect();
                (lifetimes, any_dynamized)
            }
            _ => (vec![], false),
        }
    }
}
