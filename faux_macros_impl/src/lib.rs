mod create;
mod methods;
mod self_type;

use darling::{export::NestedMeta, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;

pub fn create_impl(args: TokenStream, original: syn::ItemStruct) -> TokenStream {
    let args = match NestedMeta::parse_meta_list(args)
        .map_err(darling::Error::from)
        .and_then(|v| create::Args::from_list(&v))
    {
        Ok(v) => v,
        Err(e) => return e.write_errors(),
    };

    let mockable = create::Mockable::new(original, args);

    TokenStream::from(mockable)
}

pub fn methods_impl(args: TokenStream, original: syn::ItemImpl) -> TokenStream {
    let args = match NestedMeta::parse_meta_list(args)
        .map_err(darling::Error::from)
        .and_then(|v| methods::Args::from_list(&v))
    {
        Ok(v) => v,
        Err(e) => return e.write_errors(),
    };

    match methods::Mockable::new(original, args) {
        Ok(mockable) => TokenStream::from(mockable),
        Err(e) => e.write_errors(),
    }
}

pub fn when_impl(input: syn::Expr) -> TokenStream {
    match input {
        syn::Expr::Field(syn::ExprField {
            base,
            member: syn::Member::Named(ident),
            ..
        }) => {
            let when = quote::format_ident!("_when_{}", ident);
            quote!( { #base.#when() })
        }
        syn::Expr::MethodCall(syn::ExprMethodCall {
            receiver,
            method,
            args,
            turbofish,
            ..
        }) => {
            let when = quote::format_ident!("_when_{}", method);

            let args = args
                .into_iter()
                .map(expr_to_matcher)
                .collect::<Result<Vec<_>, _>>();

            match args {
                Err(e) => e.write_errors(),
                Ok(args) if args.is_empty() => { quote!({ #receiver.#when #turbofish() }) }
                Ok(args) => { quote!({ #receiver.#when #turbofish().with_args((#(#args,)*)) }) }
            }
        }
        expr => darling::Error::custom("faux::when! only accepts arguments in the format of: `when!(receiver.method)` or `receiver.method(args...)`")
             .with_span(&expr)
             .write_errors(),
    }
}

use quote::ToTokens;

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
