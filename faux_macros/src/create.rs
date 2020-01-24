use crate::self_type::SelfType;
use darling::FromMeta;
use quote::quote;

#[derive(Default, FromMeta)]
#[darling(default)]
pub struct Args {
    #[darling(default)]
    self_type: SelfType,
}

pub struct Mockable {
    // the real definition of the struct
    real: syn::ItemStruct,
    // the morphed definition, wraps the real struct around a MaybeFaux
    morphed: syn::ItemStruct,
}

impl Mockable {
    pub fn new(original: syn::ItemStruct, args: Args) -> Self {
        // clone original before changing anything
        let mut morphed = original.clone();

        // change the name of the original struct and then stop mutating it
        let mut real = original;
        real.ident = real_struct_new_ident(&real.ident);
        let real = real;

        // change the fields in morphed to wrap the original struct
        morphed.fields = {
            let wrapped_self = {
                let modified_name = &real.ident;
                let (_, ty_generics, _) = morphed.generics.split_for_impl();

                match args.self_type {
                    SelfType::Rc => quote! { std::rc::Rc<#modified_name #ty_generics> },
                    SelfType::Arc => quote! { std::sync::Arc<#modified_name #ty_generics> },
                    SelfType::Owned => quote! {#modified_name #ty_generics },
                    SelfType::Box => quote! { std::boxed::Box<#modified_name #ty_generics>},
                }
            };
            let vis = &morphed.vis;
            syn::Fields::Unnamed(
                syn::parse2(quote! { (#vis faux::MaybeFaux<#wrapped_self>) }).unwrap(),
            )
        };

        Mockable { real, morphed }
    }
}

impl From<Mockable> for proc_macro::TokenStream {
    fn from(mockable: Mockable) -> Self {
        let Mockable { real, morphed } = mockable;
        let (impl_generics, ty_generics, where_clause) = real.generics.split_for_impl();
        let name = &morphed.ident;

        proc_macro::TokenStream::from(quote! {
            #morphed

            impl #impl_generics #name #ty_generics #where_clause {
                pub fn faux() -> Self {
                    Self(faux::MaybeFaux::faux())
                }
            }

            #[allow(non_camel_case_types)]
            #real
        })
    }
}

pub fn real_struct_new_ident(original: &syn::Ident) -> syn::Ident {
    syn::Ident::new(&format!("_FauxOriginal_{}", original), original.span())
}
