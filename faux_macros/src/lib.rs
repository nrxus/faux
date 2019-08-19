#![feature(proc_macro_diagnostic, proc_macro_def_site)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn duck(_attrs: TokenStream, token_stream: TokenStream) -> TokenStream {
    let mut to_mock = syn::parse_macro_input!(token_stream as syn::ItemStruct);
    let vis = &to_mock.vis;
    let ident = &to_mock.ident;
    to_mock.fields.iter_mut().for_each(|f| {
        f.vis = syn::VisPublic {
            pub_token: syn::token::Pub {
                span: proc_macro2::Span::call_site(),
            },
        }
        .into()
    });

    let duck = quote! {
        #vis struct #ident(faux::MaybeQuack<_quack::#ident>);

        impl #ident {
            fn quack() -> Self {
                #ident(faux::MaybeQuack::quack())
            }
        }

        mod _quack {
            #to_mock
        }
    }
    .into();

    duck
}

#[proc_macro_attribute]
pub fn quack(_attrs: TokenStream, token_stream: TokenStream) -> TokenStream {
    let mut to_mock = syn::parse_macro_input!(token_stream as syn::ItemImpl);
    let ty = match to_mock.self_ty.as_ref() {
        syn::Type::Path(type_path) => type_path.clone(),
        _ => panic!("not supported"),
    };
    let mut moved_ty = ty.clone();
    let num_segments = ty.path.segments.len();
    moved_ty.path.segments.insert(
        num_segments - 1,
        syn::PathSegment {
            ident: syn::Ident::new("_quack", proc_macro2::Span::call_site()),
            arguments: syn::PathArguments::default(),
        },
    );

    let original = to_mock.clone();
    let self_out: syn::ReturnType = syn::parse2(quote! { -> Self }).unwrap();
    let mut mock_methods: Vec<syn::ImplItem> = to_mock
        .items
        .iter_mut()
        .filter_map(|item| match item {
            syn::ImplItem::Method(m) => Some(m),
            _ => None,
        })
        .filter_map(|mut m| {
            let args = get_ident_args(m);
            let arg_idents: Vec<_> = args.iter().map(|(ident, _)| ident).collect();
            let arg_types: Vec<_> = args.iter().map(|(_, ty)| ty).collect();
            let ident = &m.sig.ident;
            let output = &m.sig.output;
            let str_ident = ident.to_string();
            let is_mockable = args.len() != m.sig.inputs.len();
            m.block = syn::parse2(if is_mockable {
                quote! {{
                    match &self.0 {
                        faux::MaybeQuack::Quack(q) => {
                            let mut q = q.borrow_mut();
                            unsafe { q.call_mock(#str_ident, (#(#arg_idents),*)) }
                        },
                        faux::MaybeQuack::Real(r) => r.#ident(#(#arg_idents),*),
                    }
                }}
            } else {
                let body = quote! {{
                    <#moved_ty>::#ident(#(#arg_idents),*)
                }};
                let self_out = *output == self_out;
                if self_out {
                    quote! {{
                        #ty(faux::MaybeQuack::Real(#body))
                    }}
                } else {
                    body
                }
            })
                .unwrap();

            if is_mockable {
                let mock_ident = syn::Ident::new(
                    &format!("_mock_once_{}", ident),
                    proc_macro2::Span::call_site(),
                );
                let tokens = quote! {
                    pub fn #mock_ident(&mut self, mock: impl FnOnce((#(#arg_types),*)) #output + 'static) {
                        self.0.mock(#str_ident, mock);
                    }
                };

                Some(syn::parse2(tokens).unwrap())
            } else {
                None
            }
        })
        .collect();

    to_mock.items.append(&mut mock_methods);

    let quack = quote! {
        mod _real_impl {
            type #ty = super::#moved_ty;

            #original
        }

        #to_mock
    }
    .into();

    return quack;
}

fn get_ident_args<'a>(method: &mut syn::ImplItemMethod) -> Vec<(syn::Ident, syn::Type)> {
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
