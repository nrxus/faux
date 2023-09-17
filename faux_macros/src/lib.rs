extern crate proc_macro;

mod create;
mod methods;
mod self_type;

use darling::{export::NestedMeta, FromMeta};
use methods::morphed::Signature;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn faux(_args: TokenStream, original: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(original as syn::Item);
    match item {
        syn::Item::Impl(_) => todo!(),
        syn::Item::Struct(_) => todo!(),
        syn::Item::Trait(trait_) => {
            let trait_ident = &trait_.ident;
            let struct_ident = syn::Ident::new(&format!("{trait_ident}_Faux"), trait_.span());
            let (impl_generics, ty_generics, where_clause) = trait_.generics.split_for_impl();
            let methods = trait_.items.iter().filter_map(|item| match item {
                syn::TraitItem::Fn(m) => Some(m),
                _ => None,
            });

            let mut new_methods = vec![];
            let mut when_methods = vec![];

            for func in methods {
                let signature = match Signature::morph(
                    &func.sig,
                    None,
                    &syn::Visibility::Public(Default::default()),
                ) {
                    Ok(s) => s,
                    Err(e) => return e.write_errors().into(),
                };

                let block = match signature.create_body(SelfType::Owned, None) {
                    Ok(block) => block,
                    Err(e) => return e.write_errors().into(),
                };
                if let Some(methods) = signature.create_when(true) {
                    when_methods.extend(methods.into_iter().map(syn::ImplItem::Fn))
                }

                new_methods.push(syn::ImplItemFn {
                    attrs: vec![],
                    vis: syn::Visibility::Inherited,
                    defaultness: None,
                    sig: func.sig.clone(),
                    block,
                });
            }

            let extra = quote! {
                #[allow(non_camel_case_types)]
                pub struct #struct_ident(::faux::Faux);

                impl #impl_generics dyn #trait_ident #ty_generics #where_clause {
                    pub fn faux() -> #struct_ident {
                        #struct_ident(::faux::Faux::new(stringify!(trait_ident)))
                    }
                }

                #[allow(unused_variables)]
                impl #impl_generics #trait_ident #ty_generics for #struct_ident #where_clause {
                    #(#new_methods) *
                }

                impl #struct_ident {
                    #(#when_methods) *
                }
            };

            quote! {
                #trait_

                #extra
            }
            .into()
        }
        x => {
            return syn::Error::new_spanned(x, "Unsupported item wrapped by #[faux]")
                .into_compile_error()
                .into()
        }
    }
}

#[proc_macro_attribute]
pub fn create(args: TokenStream, original: TokenStream) -> TokenStream {
    let original = syn::parse_macro_input!(original as syn::ItemStruct);

    let args = match NestedMeta::parse_meta_list(args.into())
        .map_err(darling::Error::from)
        .and_then(|v| create::Args::from_list(&v))
    {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let mockable = create::Mockable::new(original, args);

    TokenStream::from(mockable)
}

#[proc_macro_attribute]
pub fn methods(args: TokenStream, original: TokenStream) -> TokenStream {
    let original = syn::parse_macro_input!(original as syn::ItemImpl);

    let args = match NestedMeta::parse_meta_list(args.into())
        .map_err(darling::Error::from)
        .and_then(|v| methods::Args::from_list(&v))
    {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    match methods::Mockable::new(original, args) {
        Ok(mockable) => TokenStream::from(mockable),
        Err(e) => e.write_errors().into(),
    }
}

#[proc_macro]
pub fn when(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse_macro_input!(input as syn::Expr) {
        syn::Expr::Field(syn::ExprField {
            base,
            member: syn::Member::Named(ident),
            ..
        }) => {
            let when = quote::format_ident!("_when_{}", ident);
            TokenStream::from(quote!( { #base.#when() }))
        }
        syn::Expr::MethodCall(syn::ExprMethodCall {
            receiver,
            method,
            args,
            ..
        }) => {
            let when = quote::format_ident!("_when_{}", method);

            let args = args
                .into_iter()
                .map(expr_to_matcher)
                .collect::<Result<Vec<_>, _>>();

            match args {
                Err(e) => e.write_errors().into(),
                Ok(args) => TokenStream::from(quote!({ #receiver.#when().with_args((#(#args,)*)) }))
            }
        }
        expr => darling::Error::custom("faux::when! only accepts arguments in the format of: `when!(receiver.method)` or `receiver.method(args...)`")
             .with_span(&expr)
             .write_errors()
             .into(),
    }
}

use quote::ToTokens;
use self_type::SelfType;
use syn::spanned::Spanned;

fn ref_matcher_maybe(
    expr: &syn::Expr,
    left: &syn::Expr,
    matcher: impl FnOnce() -> darling::Result<proc_macro2::TokenStream>,
) -> darling::Result<proc_macro2::TokenStream> {
    match left {
        syn::Expr::Infer(_) => matcher(),
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Deref(_),
            expr,
            ..
        }) => {
            let matcher = matcher()?;
            Ok(quote! { faux::matcher::ArgMatcher::<#expr>::into_ref_matcher(#matcher) })
        }
        _ => Ok(quote! { faux::matcher::eq(#expr) }),
    }
}

fn expr_to_matcher(expr: syn::Expr) -> darling::Result<proc_macro2::TokenStream> {
    match &expr {
        syn::Expr::Infer(_) => Ok(quote! { faux::matcher::any() }),
        syn::Expr::Assign(syn::ExprAssign { left, right, .. }) => {
            ref_matcher_maybe(&expr, left, || Ok(right.to_token_stream()))
        }
        syn::Expr::Binary(syn::ExprBinary {
            left, op, right, ..
        }) => ref_matcher_maybe(&expr, left, || match op {
            syn::BinOp::Eq(_) => Ok(quote! { faux::matcher::eq_against(#right) }),
            _ => Err(darling::Error::custom(format!(
                "faux:when! does not handle argument matchers with syntax: '{}'",
                expr.to_token_stream()
            ))
            .with_span(&expr)),
        }),
        arg => Ok(quote! { faux::matcher::eq(#arg) }),
    }
}
