extern crate proc_macro;

mod create;
mod methods;
mod self_type;

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;

#[proc_macro_attribute]
pub fn create(args: TokenStream, original: TokenStream) -> TokenStream {
    let original = syn::parse_macro_input!(original as syn::ItemStruct);

    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let args = match create::Args::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let mockable = create::Mockable::new(original, args);

    TokenStream::from(mockable)
}

#[proc_macro_attribute]
pub fn methods(args: TokenStream, original: TokenStream) -> TokenStream {
    let original = syn::parse_macro_input!(original as syn::ItemImpl);

    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let args = match methods::Args::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let mockable = methods::Mockable::new(original, args);

    TokenStream::from(mockable)
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
