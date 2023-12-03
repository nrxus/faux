mod morphed;
mod receiver;
mod when_arg;
mod when_output;

use std::collections::VecDeque;

use darling::FromMeta;
use morphed::Signature;
use quote::quote;
use syn::PathArguments;

use crate::{create, self_type::SelfType};

#[derive(Default, FromMeta)]
#[darling(default)]
pub struct Args {
    path: Option<syn::Path>,
    self_type: SelfType,
}

pub struct Mockable {
    // the real definitions inside the impl block
    real: syn::ItemImpl,
    // the morphed definitions
    morphed: syn::ItemImpl,
    // the _when_ methods in their own impl
    whens: syn::ItemImpl,
    // path to real struct
    real_ty: syn::TypePath,
    // path to morphed struct
    morphed_ty: syn::TypePath,
}

impl Mockable {
    pub fn new(mut original: syn::ItemImpl, args: Args) -> darling::Result<Self> {
        let (morphed_ty, real_ty) = validate(original.self_ty.as_ref(), args.path)?;
        let mut morphed = original.clone();

        let mut funcs = morphed
            .items
            .iter_mut()
            .zip(original.items.iter_mut())
            .filter_map(|(morphed, real)| match (morphed, real) {
                (syn::ImplItem::Fn(morphed), syn::ImplItem::Fn(real)) => Some((morphed, real)),
                _ => None,
            });

        let mut when_methods = vec![];
        for (morphed_func, real_func) in &mut funcs {
            normalize_idents(&mut morphed_func.sig);

            let signature = Signature::new(
                &morphed_func.sig,
                original.trait_.as_ref().map(|(_, path, _)| path),
                &morphed_func.vis,
            );

            morphed_func.block = signature.create_body(args.self_type, &real_ty, &morphed_ty)?;
            if let Some(methods) = signature.create_when() {
                unify_arguments(&mut real_func.sig);
                when_methods.extend(methods.into_iter().map(syn::ImplItem::Fn));
            }
        }

        let generics = match &morphed_ty.path.segments.last().unwrap().arguments {
            syn::PathArguments::AngleBracketed(generics_in_struct) => {
                let generics_in_struct = &generics_in_struct.args;
                let params: syn::punctuated::Punctuated<_, _> = original
                    .generics
                    .params
                    .iter()
                    .filter(|generic| match generic {
                        syn::GenericParam::Type(ty) => generics_in_struct.iter().any(|g| match g {
                            syn::GenericArgument::Type(syn::Type::Path(type_path)) => {
                                type_path.path.is_ident(&ty.ident)
                            }
                            _ => false,
                        }),
                        syn::GenericParam::Lifetime(lt) => {
                            generics_in_struct.iter().any(|g| match g {
                                syn::GenericArgument::Lifetime(lifetime_in_struct) => {
                                    lt.lifetime == *lifetime_in_struct
                                }
                                _ => false,
                            })
                        }
                        syn::GenericParam::Const(_) => true,
                    })
                    .cloned()
                    .collect();
                syn::Generics {
                    params,
                    ..original.generics.clone()
                }
            }
            _ => syn::Generics::default(),
        };

        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let whens = syn::parse_quote! {
            impl #impl_generics #morphed_ty #where_clause {
                #(#when_methods) *
            }
        };

        Ok(Mockable {
            real: original,
            morphed,
            whens,
            real_ty,
            morphed_ty,
        })
    }
}

fn unify_arguments(sig: &mut syn::Signature) {
    let mut pats = VecDeque::new();
    let mut tys = VecDeque::new();
    let mut attributes = vec![];

    while let Some(syn::FnArg::Typed(_)) = sig.inputs.last() {
        let Some(syn::FnArg::Typed(arg)) = sig.inputs.pop().map(|t| t.into_value()) else {
            unreachable!()
        };
        attributes.extend(arg.attrs);
        pats.push_front(*arg.pat);
        tys.push_front(*arg.ty);
    }

    let mut punc_pats = syn::punctuated::Punctuated::new();
    punc_pats.extend(pats);
    let mut ty_pats = syn::punctuated::Punctuated::new();
    ty_pats.extend(tys);

    if punc_pats.len() == 1 {
        sig.inputs.push(syn::FnArg::Typed(syn::PatType {
            attrs: attributes,
            pat: Box::new(punc_pats.pop().unwrap().into_value()),
            colon_token: syn::token::Colon::default(),
            ty: Box::new(ty_pats.pop().unwrap().into_value()),
        }))
    } else {
        sig.inputs.push(syn::FnArg::Typed(syn::PatType {
            attrs: attributes,
            pat: Box::new(syn::Pat::Tuple(syn::PatTuple {
                attrs: vec![],
                paren_token: syn::token::Paren::default(),
                elems: punc_pats,
            })),
            colon_token: syn::token::Colon::default(),
            ty: Box::new(syn::Type::Tuple(syn::TypeTuple {
                paren_token: syn::token::Paren::default(),
                elems: ty_pats,
            })),
        }));
    }
}

impl From<Mockable> for proc_macro::TokenStream {
    fn from(mockable: Mockable) -> Self {
        let Mockable {
            real,
            morphed,
            whens,
            mut real_ty,
            morphed_ty,
        } = mockable;

        // create an identifier for the mod containing the real implementation
        // this is necessary until we are allowed to introduce type aliases within impl blocks
        let mod_ident = {
            let uuid = uuid::Uuid::new_v4();
            let ident = &real_ty.path.segments.last().unwrap().ident;
            syn::Ident::new(
                &match &real.trait_ {
                    None => format!("_faux_real_impl_{}_{}", ident, uuid.simple()),
                    Some((_, trait_, _)) => format!(
                        "_faux_real_impl_{}_{}_{}",
                        ident,
                        trait_.segments.last().unwrap().ident,
                        uuid.simple()
                    ),
                },
                proc_macro2::Span::call_site(),
            )
        };

        // make the original methods at least pub(super)
        // since they will be hidden in a nested mod
        let mut real = real;
        if real.trait_.is_none() {
            publicize_methods(&mut real);
        }
        let real = real;

        // creates an alias `type Foo = path::to::RealFoo` that may be wrapped inside some mods
        let alias = {
            let mut path_to_ty = morphed_ty.path.segments;
            let path_to_real_from_alias_mod = {
                // let mut real_ty = real_ty.clone();
                real_ty.path.segments.last_mut().unwrap().arguments = PathArguments::None;
                let first_path = &real_ty.path.segments.first().unwrap().ident;
                if *first_path == syn::Ident::new("crate", first_path.span()) {
                    // if it is an absolute position then no need to "super" up to find it
                    quote! { #real_ty }
                } else {
                    // otherwise do as many supers until you find the real struct definition
                    // one extra super for the nested mod
                    let supers = std::iter::repeat(quote! { super }).take(path_to_ty.len());
                    quote! { #(#supers::)*#real_ty }
                }
            };

            // type Foo = path::to::RealFoo
            let alias = {
                // the alias has to be public up to the mod wrapping it
                let pub_supers = if path_to_ty.len() < 2 {
                    quote! {}
                } else {
                    let supers = std::iter::repeat(quote! { super }).take(path_to_ty.len() - 1);
                    quote! { pub(#(#supers)::*) }
                };
                let pathless_type = path_to_ty.pop().unwrap();
                let ident = &pathless_type.value().ident;
                quote! {
                    //do not warn for things like Foo<i32> = RealFoo<i32>
                    #[allow(non_camel_case_types)]
                    #[allow(clippy::builtin_type_shadow)]
                    #pub_supers use #path_to_real_from_alias_mod as #ident;
                }
            };

            // nest the type alias in load-bearing mods
            path_to_ty.into_iter().fold(alias, |alias, segment| {
                quote! { mod #segment { #alias } }
            })
        };

        proc_macro::TokenStream::from(quote! {
            #morphed

            #whens

            mod #mod_ident {
                // make everything that was in-scope above also in-scope in this mod
                use super::*;

                #alias

                #real
            }
        })
    }
}

fn real_ty_path(mock_ty: &syn::TypePath, mod_path: Option<syn::Path>) -> syn::TypePath {
    let type_ident = &mock_ty.path.segments.last().unwrap().ident;
    let mut real_ty = mock_ty.clone();

    // combine a passed in path if given one
    // this will find the full path from the impl block to the morphed struct
    if let Some(mod_path) = mod_path {
        let path = std::mem::replace(&mut real_ty.path.segments, mod_path.segments);
        real_ty.path.segments.extend(path);
    }

    // now replace the last path segment with the original struct ident
    // this is now the path to the real struct from the impl block
    real_ty.path.segments.last_mut().unwrap().ident = create::real_struct_new_ident(type_ident);
    real_ty
}

// makes methods in this impl block be at least visible to super
fn publicize_methods(impl_block: &mut syn::ItemImpl) {
    impl_block
        .items
        .iter_mut()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(m) => Some(m),
            _ => None,
        })
        .filter(|method| method.vis == syn::Visibility::Inherited)
        .for_each(|method| method.vis = syn::parse_quote! { pub(super) });
}

fn normalize_idents(signature: &mut syn::Signature) {
    signature
        .inputs
        .iter_mut()
        .filter_map(|a| match a {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(arg) => Some(arg.pat.as_mut()),
        })
        .enumerate()
        .for_each(|(i, arg_pat)| match arg_pat {
            syn::Pat::Ident(pat_ident) => {
                pat_ident.by_ref = None;
                pat_ident.mutability = None;
                pat_ident.subpat = None;
            }
            non_ident => {
                let old = std::mem::replace(
                    non_ident,
                    syn::Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        subpat: None,
                        ident: syn::Ident::new(
                            &format!("_faux_arg_{i}"),
                            proc_macro2::Span::call_site(),
                        ),
                    }),
                );

                let attrs = match old {
                    syn::Pat::Const(p) => p.attrs,
                    syn::Pat::Ident(p) => p.attrs,
                    syn::Pat::Lit(p) => p.attrs,
                    syn::Pat::Macro(p) => p.attrs,
                    syn::Pat::Or(p) => p.attrs,
                    syn::Pat::Paren(p) => p.attrs,
                    syn::Pat::Path(p) => p.attrs,
                    syn::Pat::Range(p) => p.attrs,
                    syn::Pat::Reference(p) => p.attrs,
                    syn::Pat::Rest(p) => p.attrs,
                    syn::Pat::Slice(p) => p.attrs,
                    syn::Pat::Struct(p) => p.attrs,
                    syn::Pat::Tuple(p) => p.attrs,
                    syn::Pat::TupleStruct(p) => p.attrs,
                    syn::Pat::Type(p) => p.attrs,
                    syn::Pat::Wild(p) => p.attrs,
                    _ => vec![],
                };

                let syn::Pat::Ident(new) = non_ident else {
                    unreachable!()
                };
                new.attrs = attrs;
            }
        });
}

fn validate(
    ty: &syn::Type,
    real_ty_mod: Option<syn::Path>,
) -> darling::Result<(syn::TypePath, syn::TypePath)> {
    // check that we can support this impl
    let morphed_ty = match ty {
        syn::Type::Path(type_path) => type_path.clone(),
        node => {
            return Err(darling::Error::custom(
                "#[faux::methods] does not support implementing types that are not a simple path",
            )
            .with_span(node));
        }
    };

    if let Some(segment) = morphed_ty
        .path
        .segments
        .iter()
        .find(|segment| segment.ident == "crate" || segment.ident == "super")
    {
        return Err(
                darling::Error::custom(
                    format!("#[faux::methods] does not support implementing types with '{segment}' in the path. Consider importing one level past '{segment}' and using #[faux::methods(path = \"{segment}\")]", segment = segment.ident)
                ).with_span(segment)
            );
    }

    // start transforming
    let real_ty = real_ty_path(&morphed_ty, real_ty_mod);

    Ok((morphed_ty, real_ty))
}
