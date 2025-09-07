use faux_macros_impl::{create_impl, methods_impl, when_impl};
use proc_macro2::TokenStream;

#[proc_macro_attribute]
pub fn create(args: proc_macro::TokenStream, original: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let original = syn::parse_macro_input!(original as syn::ItemStruct);

    let result = create_impl(
        proc_macro2::TokenStream::from(args), 
        original);

    proc_macro::TokenStream::from(result)
}

#[proc_macro_attribute]
pub fn methods(args: proc_macro::TokenStream, original: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let original = syn::parse_macro_input!(original as syn::ItemImpl);

    let result = methods_impl(
        TokenStream::from(args), 
        original);

    proc_macro::TokenStream::from(result)
}

#[proc_macro]
pub fn when(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::Expr);

    let result = when_impl(input);

    proc_macro::TokenStream::from(result)
}