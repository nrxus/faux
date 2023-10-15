use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

#[derive(Debug)]
pub struct WhenArg {
    pub ty: syn::Type,
    pub lifetimes: Vec<syn::Lifetime>,
    pub dynamized: bool,
}

impl WhenArg {
    pub fn new(mut ty: syn::Type, arg_num: usize) -> Self {
        let (lifetimes, dynamized) = Self::sanitize(&mut ty, (arg_num, &mut 0));
        WhenArg {
            ty,
            lifetimes,
            dynamized,
        }
    }

    fn sanitize(
        ty: &mut syn::Type,
        lifetime_id: (usize, &mut usize),
    ) -> (Vec<syn::Lifetime>, bool) {
        fn new_lt(lifetime_id: (usize, &mut usize), span: Span) -> syn::Lifetime {
            let new_lt = format!("'_faux_arg_{}_{}", lifetime_id.0, lifetime_id.1);
            *lifetime_id.1 += 1;
            syn::Lifetime::new(&new_lt, span)
        }

        match ty {
            syn::Type::Array(syn::TypeArray { elem, .. })
            | syn::Type::Group(syn::TypeGroup { elem, .. })
            | syn::Type::Paren(syn::TypeParen { elem, .. })
            | syn::Type::Ptr(syn::TypePtr { elem, .. })
            | syn::Type::Slice(syn::TypeSlice { elem, .. }) => Self::sanitize(elem, lifetime_id),
            syn::Type::ImplTrait(t) => {
                let span = t.span();
                let mut new_lifetimes = vec![];
                let lifetimes = t.bounds.iter_mut().filter_map(|b| match b {
                    syn::TypeParamBound::Lifetime(lt) => Some(lt),
                    _ => None,
                });
                let mut any_lifetime = false;
                for lt in lifetimes {
                    any_lifetime = true;
                    if lt.ident == "_" {
                        *lt = new_lt((lifetime_id.0, lifetime_id.1), span);
                        new_lifetimes.push(lt.clone());
                    }
                }

                if !any_lifetime {
                    let new_lt = new_lt(lifetime_id, span);
                    t.bounds.push(syn::TypeParamBound::Lifetime(new_lt.clone()));
                    new_lifetimes.push(new_lt);
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
                            let new_lt = new_lt((lifetime_id.0, lifetime_id.1), lt.span());
                            *lt = new_lt.clone();
                            joined_lifetimes.push(new_lt);
                        }
                        syn::GenericArgument::Type(ty) => {
                            let (lifetimes, dynamized) =
                                Self::sanitize(ty, (lifetime_id.0, lifetime_id.1));
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
                    Self::sanitize(&mut ty.elem, lifetime_id)
                } else {
                    let new_lt = new_lt((lifetime_id.0, lifetime_id.1), ty.elem.span());
                    ty.lifetime = Some(new_lt.clone());
                    let (mut lifetimes, dynamized) = Self::sanitize(&mut ty.elem, lifetime_id);
                    lifetimes.push(new_lt);
                    (lifetimes, dynamized)
                }
            }
            syn::Type::Tuple(t) => {
                let mut any_dynamized = false;
                let lifetimes = t
                    .elems
                    .iter_mut()
                    .flat_map(|ty| {
                        let (lts, dynamized) = Self::sanitize(ty, (lifetime_id.0, lifetime_id.1));
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

impl ToTokens for WhenArg {
    fn to_tokens(&self, token_stream: &mut proc_macro2::TokenStream) {
        if self.dynamized {
            let ty = &self.ty;
            token_stream.extend(quote! { std::boxed::Box<#ty> })
        } else {
            self.ty.to_tokens(token_stream)
        }
    }
}
