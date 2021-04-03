extern crate proc_macro;

mod create;
mod methods;
mod self_type;

use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

enum Item {
    Trait(syn::ItemTrait),
    Struct(syn::ItemStruct),
}

#[proc_macro_attribute]
pub fn create(args: TokenStream, original: TokenStream) -> TokenStream {
    let original = syn::parse_macro_input!(original as syn::Item);

    let original = match original {
        syn::Item::Struct(s) => Item::Struct(s),
        syn::Item::Trait(t) => Item::Trait(t),
        _ => {
            return TokenStream::from(
                darling::Error::custom("only allowed on structs or traits")
                    .with_span(&original)
                    .write_errors(),
            )
        }
    };

    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let args = match create::Args::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    match original {
        Item::Struct(s) => create::Mockable::new(s, args).into(),
        Item::Trait(t) => create::MockableTrait::new(t).into(),
    }
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

fn ref_matcher_maybe(
    expr: &syn::Expr,
    left: &syn::Expr,
    matcher: impl FnOnce() -> darling::Result<proc_macro2::TokenStream>,
) -> darling::Result<proc_macro2::TokenStream> {
    match left {
        syn::Expr::Verbatim(t) if t.to_string() == "_" => matcher(),
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
        syn::Expr::Verbatim(t) if t.to_string() == "_" => Ok(quote! { faux::matcher::any() }),
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
