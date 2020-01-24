mod morphed;

use crate::{create, self_type::SelfType};
use darling::FromMeta;
use morphed::Signature;
use quote::quote;

#[derive(Default, FromMeta)]
#[darling(default)]
pub struct Args {
    path: Option<syn::Path>,
    self_type: SelfType,
}

pub struct Mockable {
    // the real definitions inside the impl block
    real: syn::ItemImpl,
    // the morphed definitions and the _where_ mehods
    morphed_and_wheres: syn::ItemImpl,
    // path to real struct
    real_ty: syn::TypePath,
    // path to morphed struct
    morphed_ty: syn::TypePath,
}

impl Mockable {
    pub fn new(real: syn::ItemImpl, args: Args) -> Self {
        // check that we can support this impl
        let morphed_ty = match real.self_ty.as_ref() {
            syn::Type::Path(type_path) => type_path.clone(),
            _ => panic!(
                "#[faux::methods] does not support implementing types that are not a simple path"
            ),
        };

        if let Some(segment) = morphed_ty
            .path
            .segments
            .iter()
            .find(|segment| segment.ident == "crate" || segment.ident == "super")
        {
            panic!("#[faux::methods] does not support implemeneting types with '{segment}' in the path. Consider importing one level past '{segment}' and using #[faux::methods(path = \"{segment}\")]", segment = segment.ident);
        }

        // start transforming
        let real_ty = real_ty(&morphed_ty, args.path);

        let mut morphed_and_wheres = real.clone();

        let mut methods = morphed_and_wheres
            .items
            .iter_mut()
            .filter_map(|item| match item {
                syn::ImplItem::Method(m) => Some(m),
                _ => None,
            });

        let mut when_methods = vec![];
        for func in &mut methods {
            let signature = Signature::morph(&mut func.sig);
            func.block = signature.create_body(&args.self_type, &real_ty, &morphed_ty);
            if let Some(when_method) = signature.create_when() {
                when_methods.push(syn::ImplItem::Method(when_method));
            }
        }

        morphed_and_wheres.items.extend(when_methods);

        Mockable {
            real,
            morphed_and_wheres,
            real_ty,
            morphed_ty,
        }
    }
}

impl From<Mockable> for proc_macro::TokenStream {
    fn from(mockable: Mockable) -> Self {
        let Mockable {
            real,
            morphed_and_wheres,
            real_ty,
            morphed_ty,
        } = mockable;

        // create an identifier for the mod containing the real implementation
        // this is necessary until we are allowed to introduce type aliases within impl blocks
        let mod_ident = {
            let ident = &real_ty.path.segments.last().unwrap().ident;
            syn::Ident::new(
                &format!("_faux_real_impl_{}", ident),
                proc_macro2::Span::call_site(),
            )
        };

        // make the original methods at least pub(super)
        // since they will be hidden in a nested mod
        let mut real = real;
        publicize_methods(&mut real);
        let real = real;

        // creates an alias `type Foo = path::to::RealFoo` that may be wrapped inside some mods
        let alias = {
            let mut path_to_ty = morphed_ty.path.segments;
            let path_to_real_from_alias_mod = {
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
                let pathless_type = path_to_ty.pop().unwrap();
                quote! {
                    pub type #pathless_type = #path_to_real_from_alias_mod;
                }
            };

            // nest the type alias in load-bearing mods
            path_to_ty.into_iter().fold(alias, |alias, segment| {
                quote! { mod #segment { #alias } }
            })
        };

        proc_macro::TokenStream::from(quote! {
            #morphed_and_wheres

            mod #mod_ident {
                // make everything that was in-scope above also in-scope in this mod
                use super::*;

                #alias

                #real
            }
        })
    }
}

fn real_ty(morphed_ty: &syn::TypePath, path: Option<syn::Path>) -> syn::TypePath {
    let type_ident = &morphed_ty.path.segments.last().unwrap().ident;
    // combine a passed in path if given one
    // this will find the full path from the impl block to the morphed struct
    let mut path_to_morph_from_here = match path {
        None => morphed_ty.clone(),
        Some(mut path) => {
            let morphed_ty = morphed_ty.clone();
            path.segments.extend(morphed_ty.path.segments);
            syn::TypePath { path, ..morphed_ty }
        }
    };

    // now replace the last path segment with the original struct ident
    // this is now the path to the real struct from the impl block
    path_to_morph_from_here
        .path
        .segments
        .last_mut()
        .unwrap()
        .ident = create::real_struct_new_ident(type_ident);
    path_to_morph_from_here
}

// makes methods in this impl block be at least visible to super
fn publicize_methods(impl_block: &mut syn::ItemImpl) {
    impl_block
        .items
        .iter_mut()
        .filter_map(|item| match item {
            syn::ImplItem::Method(m) => Some(m),
            _ => None,
        })
        .filter(|method| method.vis == syn::Visibility::Inherited)
        .for_each(|mut method| {
            method.vis = syn::parse2(quote! { pub(super) }).unwrap();
        });
}
