extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::{quote, ToTokens};

#[proc_macro_attribute]
pub fn create(_attrs: TokenStream, token_stream: TokenStream) -> TokenStream {
    let mut original = syn::parse_macro_input!(token_stream as syn::ItemStruct);
    let mut mock_version = original.clone();
    let original_name = original.ident.clone();
    let (impl_generics, ty_generics, where_clause) = original.generics.split_for_impl();

    original.ident = original_struct_ident(&original_name);

    let modified_name = &original.ident;
    let original_vis = &original.vis;
    mock_version.fields = syn::Fields::Unnamed(
        syn::parse2(quote! { (#original_vis faux::MaybeFaux<#modified_name #ty_generics>) })
            .unwrap(),
    );

    TokenStream::from(quote! {
        #mock_version

        impl #impl_generics #original_name #ty_generics #where_clause {
            pub fn faux() -> Self {
                Self(faux::MaybeFaux::faux())
            }
        }

        #[allow(non_camel_case_types)]
        #original
    })
}

#[proc_macro_attribute]
pub fn methods(attrs: TokenStream, token_stream: TokenStream) -> TokenStream {
    let mut impl_block = syn::parse_macro_input!(token_stream as syn::ItemImpl);

    if let Some(_) = impl_block.trait_ {
        panic!("#[faux::methods] does no support trait implementations")
    }

    let ty = match impl_block.self_ty.as_ref() {
        syn::Type::Path(type_path) => type_path,
        _ => panic!(
            "#[faux::methods] does not support implementing types that are not a simple path"
        ),
    };

    let ident = &ty.path.segments.last().unwrap().ident;

    let original_struct = {
        let mut ty = if attrs.is_empty() {
            ty.clone()
        } else {
            let ty = ty.clone();
            let mut path = syn::parse_macro_input!(attrs as syn::Path);
            path.segments.extend(ty.path.segments);
            syn::TypePath { path, ..ty }
        };

        ty.path.segments.last_mut().unwrap().ident = original_struct_ident(ident);
        ty
    };

    let mut original = impl_block.clone();
    publicize_methods(&mut original);

    let self_return: syn::ReturnType = syn::parse2(quote! { -> Self}).unwrap();
    let ty_return: syn::ReturnType = syn::parse2(quote! { -> #ty }).unwrap();
    let ignore_unused_mut_attr = syn::Attribute {
        pound_token: Default::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Default::default(),
        path: syn::parse2(quote! { allow }).unwrap(),
        tokens: "(unused_mut)".parse().unwrap(),
    };

    let mut when_methods: Vec<syn::ImplItem> = impl_block
        .items
        .iter_mut()
        .filter_map(|item| match item {
            syn::ImplItem::Method(m) => Some(m),
            _ => None,
        })
        .filter_map(|mut m| {
            // in case a method has `mut self` as a parameter since we
            // are not modifying self directly; only proxying to
            // either mock or real instance
            m.attrs.push(ignore_unused_mut_attr.clone());
            let is_async = m.sig.asyncness.is_some();
            let args = method_args(m);
            let arg_idents: Vec<_> = args.iter().map(|(ident, _)| ident).collect();
            let arg_types: Vec<_> = args.iter().map(|(_, ty)| ty).collect();
            let ident = &m.sig.ident;
            let output = &m.sig.output;
            let error_msg = format!(
                "'{}::{}' is not mocked",
                ty.to_token_stream(),
                ident
            );
            let is_method = args.len() != m.sig.inputs.len();
            let is_private = m.vis == syn::Visibility::Inherited;
            let returns_self = m.sig.output == self_return || m.sig.output == ty_return;
            let method_name = format!("{}", ident);
            let mut block = if !is_method {
                // associated function; cannot be mocked
                // proxy to real associated function
                let mut inner_body = quote! { <#original_struct>::#ident(#(#arg_idents),*) };
                if is_async {
                    inner_body = quote! { #inner_body.await }
                }
                quote! {{ #inner_body }}
            } else {
                let call_mock = if is_private {
                    quote! {
                        panic!("attempted to call private method on mocked instance")
                    }
                } else {
                    quote! {
                        let mut q = q.try_lock().unwrap();
                        unsafe {
                            q.get_mock(#method_name).expect(#error_msg).call((#(#arg_idents),*))
                        }
                    }
                };
                let mut proxy_real = quote! { r.#ident(#(#arg_idents),*) };
                if is_async {
                    proxy_real = quote! { #proxy_real.await }
                }
                quote! {{
                    match self {
                        Self(faux::MaybeFaux::Real(r)) => { #proxy_real },
                        Self(faux::MaybeFaux::Faux(q)) => { #call_mock },
                    }
                }}
            };

            // wrap inside MaybeFaux if we are returning ourselves
            if returns_self {
                block = quote! {{ Self(faux::MaybeFaux::Real(#block)) }}
            }

            m.block = syn::parse2(block).unwrap();

            // return _when_{} methods for all mocked methods
            if is_method && !is_private {
                let mock_ident = syn::Ident::new(
                    &format!("_when_{}", ident),
                    proc_macro2::Span::call_site(),
                );
                let empty = Box::new( syn::parse2(quote! { () }) .unwrap());
                let output = match output {
                    syn::ReturnType::Default => &empty,
                    syn::ReturnType::Type(_, ty) => ty,
                };
                let tokens = quote! {
                    pub fn #mock_ident(&mut self) -> faux::When<(#(#arg_types),*), #output> {
                        match &mut self.0 {
                            faux::MaybeFaux::Faux(faux) => faux::When::new(
                                #method_name,
                                faux.get_mut().unwrap()
                            ),
                            faux::MaybeFaux::Real(_) => panic!("not allowed to mock a real instance!"),
                        }
                    }
                };

                Some(syn::parse2(tokens).unwrap())
            } else {
                None
            }
        })
        .collect();

    impl_block.items.append(&mut when_methods);

    let mod_ident = syn::Ident::new(
        &format!("_faux_real_impl_{}", ident),
        proc_macro2::Span::call_site(),
    );

    let first_path = &original_struct.path.segments.first().unwrap().ident;

    let alias_path = if *first_path == syn::Ident::new("crate", first_path.span()) {
        quote! { #original_struct }
    } else {
        quote! { super::#original_struct }
    };

    TokenStream::from(quote! {
    #impl_block

    mod #mod_ident {
            use super::*;

            type #ty = #alias_path;

            #original
    }
    })
}

fn method_args(method: &mut syn::ImplItemMethod) -> Vec<(syn::Ident, syn::Type)> {
    method
        .sig
        .inputs
        .iter_mut()
        .filter_map(|a| match a {
            syn::FnArg::Typed(arg) => Some(arg),
            _ => None,
        })
        .enumerate()
        .map(|(index, arg)| {
            let ident = syn::Ident::new(&format!("_x{}", index), proc_macro2::Span::call_site());
            arg.pat = Box::new(syn::Pat::Ident(syn::PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                subpat: None,
                ident: ident.clone(),
            }));
            (ident, *arg.ty.clone())
        })
        .collect()
}

#[proc_macro_hack]
pub fn when(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expr = syn::parse_macro_input!(input as syn::ExprField);
    let base = expr.base;
    let method = match expr.member {
        syn::Member::Named(ident) => ident,
        syn::Member::Unnamed(_) => panic!("not a method call"),
    };
    let when = syn::Ident::new(&format!("_when_{}", method), proc_macro2::Span::call_site());

    TokenStream::from(quote!( { #base.#when() }))
}

fn original_struct_ident(original: &syn::Ident) -> syn::Ident {
    syn::Ident::new(&format!("_FauxOriginal_{}", original), original.span())
}

fn publicize_methods(impl_block: &mut syn::ItemImpl) {
    impl_block
        .items
        .iter_mut()
        .filter_map(|item| match item {
            syn::ImplItem::Method(m) => Some(m),
            _ => None,
        })
        .for_each(|mut method| {
            method.vis = syn::parse2(quote! { pub }).unwrap();
        });
}
