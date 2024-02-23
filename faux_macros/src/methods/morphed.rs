use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, punctuated::Punctuated, spanned::Spanned, PathArguments};

use crate::{
    methods::{when_arg::WhenArg, when_output::WhenOutput},
    self_type::SelfType,
};

use super::receiver::SelfKind;

pub struct Signature<'a> {
    name: &'a syn::Ident,
    args: Vec<&'a syn::Pat>,
    is_async: bool,
    output: Option<&'a syn::Type>,
    method_data: Option<MethodData>,
    qualified_real_fn: syn::ExprPath,
}

pub struct MethodData {
    self_kind: SelfKind,
    mock_data: Option<MockData>,
}

pub struct MockData {
    self_ty: Box<syn::Type>,
    arg_types: Vec<WhenArg>,
    generics: syn::Generics,
    output: WhenOutput,
}

impl MockData {
    pub fn new(mut self_ty: Box<syn::Type>, signature: &syn::Signature) -> Self {
        let imp_lt = implicit_ref_lifetime(&mut self_ty);

        let mut arg_types: Vec<_> = signature
            .inputs
            .iter()
            .skip(1)
            .enumerate()
            .map(|(i, a)| match a {
                syn::FnArg::Typed(arg) => WhenArg::new((arg.ty.as_ref()).clone(), i),
                syn::FnArg::Receiver(_) => {
                    unreachable!("faux: this is a weird bug if you reached this")
                }
            })
            .collect();

        let mut generics = signature.generics.clone();
        let imp_lt = imp_lt.or_else(|| {
            let mut possible_lifetimes = arg_types
                .iter_mut()
                .filter_map(|t| implicit_ref_lifetime(&mut t.ty));
            let first = possible_lifetimes.next()?;
            if possible_lifetimes.next().is_some() {
                None
            } else {
                Some(first)
            }
        });

        if let Some((imp_lt, true)) = &imp_lt {
            generics
                .params
                .push(syn::GenericParam::Lifetime(syn::LifetimeParam::new(
                    imp_lt.clone(),
                )));
        }

        let output = WhenOutput::new(signature.output.clone(), imp_lt.as_ref().map(|(l, _)| l));

        MockData {
            self_ty,
            arg_types,
            output,
            generics,
        }
    }
}

fn swap_types(ty: &mut syn::Type, old_ty: &syn::TypePath, new_ty: &syn::TypePath) {
    match ty {
        syn::Type::Path(syn::TypePath { path, .. }) => {
            if path == &old_ty.path || path.is_ident("Self") {
                *path = new_ty.path.clone();
            }
            match &mut path.segments.last_mut().unwrap().arguments {
                PathArguments::None => {}
                PathArguments::AngleBracketed(args) => {
                    args.args.iter_mut().for_each(|arg| match arg {
                        syn::GenericArgument::Type(ty)
                        | syn::GenericArgument::AssocType(syn::AssocType { ty, .. }) => {
                            swap_types(ty, old_ty, new_ty)
                        }
                        _ => {}
                    });
                }
                PathArguments::Parenthesized(args) => {
                    match &mut args.output {
                        syn::ReturnType::Default => {}
                        syn::ReturnType::Type(_, output) => swap_types(output, old_ty, new_ty),
                    }
                    args.inputs
                        .iter_mut()
                        .for_each(|i| swap_types(i, old_ty, new_ty));
                }
            }
        }
        syn::Type::Reference(syn::TypeReference { elem, .. })
        | syn::Type::Array(syn::TypeArray { elem, .. })
        | syn::Type::Group(syn::TypeGroup { elem, .. })
        | syn::Type::Paren(syn::TypeParen { elem, .. })
        | syn::Type::Ptr(syn::TypePtr { elem, .. })
        | syn::Type::Slice(syn::TypeSlice { elem, .. }) => swap_types(elem, old_ty, new_ty),
        syn::Type::BareFn(syn::TypeBareFn { inputs, output, .. }) => {
            match output {
                syn::ReturnType::Default => {}
                syn::ReturnType::Type(_, output) => swap_types(output, old_ty, new_ty),
            }
            inputs
                .iter_mut()
                .for_each(|i| swap_types(&mut i.ty, old_ty, new_ty));
        }
        syn::Type::Tuple(syn::TypeTuple { elems, .. }) => {
            elems.iter_mut().for_each(|i| swap_types(i, old_ty, new_ty));
        }
        _ => {}
    }
}

impl<'a> Signature<'a> {
    pub fn new(
        signature: &'a syn::Signature,
        trait_path: Option<&'a syn::Path>,
        vis: &syn::Visibility,
        mocked_ty: &syn::TypePath,
        real_ty: &syn::TypePath,
    ) -> Signature<'a> {
        let output = match &signature.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.as_ref()),
        };

        let receiver = signature.inputs.first().and_then(|i| match i {
            syn::FnArg::Receiver(r) => Some(r),
            syn::FnArg::Typed(_) => None,
        });

        let name = &signature.ident;

        let mut qualified_real_fn: Option<syn::ExprPath> = None;
        let method_data = receiver.map(|receiver| {
            let self_kind = SelfKind::new(receiver);
            let is_private = trait_path.is_none() && *vis == syn::Visibility::Inherited;
            let mock_data = if is_private {
                None
            } else {
                let ident = syn::Ident::new(&format!("_faux_{name}"), name.span());
                qualified_real_fn = Some(parse_quote! { <#real_ty>::#ident });
                let mut self_ty = receiver.ty.clone();
                swap_types(&mut self_ty, mocked_ty, real_ty);
                Some(MockData::new(self_ty, signature))
            };

            MethodData {
                self_kind,
                mock_data,
            }
        });

        let qualified_real_fn = qualified_real_fn.unwrap_or_else(|| match trait_path {
            None => parse_quote! { <#real_ty>::#name },
            Some(trait_path) => parse_quote! { <#real_ty as #trait_path>::#name },
        });

        Signature {
            name,
            args: signature
                .inputs
                .iter()
                .skip(method_data.is_some() as usize)
                .map(|a| match a {
                    syn::FnArg::Typed(arg) => &*arg.pat,
                    syn::FnArg::Receiver(_) => {
                        unreachable!("this is a weird bug in faux if you reached this")
                    }
                })
                .collect(),
            is_async: signature.asyncness.is_some(),
            output,
            method_data,
            qualified_real_fn,
        }
    }

    pub fn create_body(
        &self,
        real_self: SelfType,
        morphed_ty: &syn::TypePath,
    ) -> darling::Result<syn::Block> {
        let ret = self.create_body_inner(real_self, morphed_ty)?;

        Ok(syn::Block {
            stmts: vec![syn::Stmt::Expr(ret, None)],
            brace_token: Default::default(),
        })
    }

    fn create_body_inner(
        &self,
        real_self: SelfType,
        morphed_ty: &syn::TypePath,
    ) -> darling::Result<syn::Expr> {
        let method_data = match &self.method_data {
            // not a method - proxy to real as-is
            None => {
                let proxy_real =
                    self.proxy_real(real_self, morphed_ty, self.args.iter().copied())?;

                return Ok(syn::parse2(proxy_real).unwrap());
            }
            // else we can either proxy for real instances
            // or call the mock store for faux instances
            Some(method_data) => method_data,
        };

        // need to pass the real Self arg to the real method
        let real_self_arg = syn::Pat::Ident(syn::PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: syn::Ident::new("_maybe_faux_real", proc_macro2::Span::call_site()),
            subpat: None,
        });

        let mock_data = match &method_data.mock_data {
            None => {
                let proxy_real = self.proxy_real(
                    real_self,
                    morphed_ty,
                    std::iter::once(&real_self_arg).chain(self.args.iter().copied()),
                )?;

                return Ok(syn::parse_quote! {{
                    let wrapped = ::faux::MockWrapper::inner(self);
                    let _maybe_faux_real = match ::faux::FauxCaller::try_into_real(wrapped) {
                        Some(r) => r,
                        None => panic!("faux error: private methods are not stubbable; and therefore not directly callable in a mock"),
                    };

                    #proxy_real
                }});
            }
            Some(mock_data) => mock_data,
        };

        let mut args = self
            .args
            .iter()
            .zip(mock_data.arg_types.iter())
            .map(|(ident, when_arg)| {
                if when_arg.dynamized {
                    quote! {
                        std::boxed::Box::new(#ident)
                    }
                } else {
                    quote! { #ident }
                }
            });

        let args: syn::Pat = if args.len() == 1 {
            let arg = args.next().unwrap();
            syn::parse_quote! { #arg }
        } else {
            syn::parse_quote! { (#(#args,)*) }
        };

        // let mut proxy_real = self.proxy_real(
        //     real_self,
        //     real_fn,
        //     morphed_ty,
        //     [&real_self_arg, &args].into_iter(),
        // )?;

        // if mock_data.output.dynamized {
        //     proxy_real = quote! { std::boxed::Box::new(#proxy_real) };
        // }

        // let faux_ident =
        // syn::Ident::new(&format!("_faux_{}", name), proc_macro2::Span::call_site());
        // let call_stub = quote! { wrapper.#faux_ident(#args) };

        let real_fn = &self.qualified_real_fn;
        let name = &self.name;
        Ok(syn::parse_quote! {{
            use ::faux::MockWrapper;
            use ::faux::FauxCaller;

            let inner = self.inner();
            inner.call(#real_fn, stringify!(#name), #args)
        }})
        // Ok(method_data.self_kind.method_body(proxy_real, call_stub))
    }

    fn proxy_real<'p>(
        &self,
        real_self: SelfType,
        morphed_ty: &syn::TypePath,
        args: impl Iterator<Item = &'p syn::Pat>,
    ) -> darling::Result<TokenStream> {
        let proxy = &self.qualified_real_fn;
        let mut proxy = quote! { #proxy(#(#args),*) };
        if self.is_async {
            proxy.extend(quote! { .await })
        }

        self.wrap_self(morphed_ty, real_self, &proxy)
            .map(|p| p.unwrap_or(proxy))
    }

    pub fn create_when(&self) -> Option<syn::ImplItemFn> {
        self.method_data
            .as_ref()
            .and_then(|m| m.mock_data.as_ref())
            .map(|m| m.create_when(self.name, &self.qualified_real_fn))
    }

    fn wrap_self(
        &self,
        morphed_ty: &syn::TypePath,
        real_self: SelfType,
        block: &TokenStream,
    ) -> darling::Result<Option<TokenStream>> {
        // TODO: use let-else once we bump MSRV to 1.65.0
        let output = match self.output {
            Some(output) => output,
            None => return Ok(None),
        };
        if !contains_self(output, morphed_ty) {
            return Ok(None);
        }

        let is_self = |ty: &syn::TypePath| {
            ty == morphed_ty || (ty.qself.is_none() && ty.path.is_ident("Self"))
        };

        let output = match output {
            syn::Type::Path(output) => output,
            output => return Err(unhandled_self_return(output)),
        };

        let wrapped = if is_self(output) {
            match real_self {
                SelfType::Owned => quote! { Self(faux::MaybeFaux::Real(#block)) },
                generic => {
                    let new_path = generic
                        .new_path()
                        .expect("Generic self should have new() function");
                    quote! { Self(faux::MaybeFaux::Real(#new_path(#block))) }
                }
            }
        } else {
            let unpathed_output = output.path.segments.last().unwrap();
            let generics = match &unpathed_output.arguments {
                syn::PathArguments::AngleBracketed(args) => args,
                g => return Err(unhandled_self_return(g)),
            };
            let first_arg = generics
                .args
                .first()
                .expect("faux bug: no generic arguments but expected at least one");
            let first_arg = match first_arg {
                syn::GenericArgument::Type(syn::Type::Path(ty)) => ty,
                _ => return Err(unhandled_self_return(generics)),
            };

            if !is_self(first_arg) {
                return Err(unhandled_self_return(generics));
            }

            let output_ident = &unpathed_output.ident;
            match real_self {
                SelfType::Rc if output_ident == "Rc" => {
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(#block))) }
                }
                SelfType::Arc if output_ident == "Arc" => {
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(#block))) }
                }
                SelfType::Box if output_ident == "Box" => {
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(#block))) }
                }
                SelfType::Owned if output_ident == "Result" || output_ident == "Option" => {
                    quote! { { #block }.map(faux::MaybeFaux::Real).map(Self) }
                }
                SelfType::Owned if output_ident == "Box" => {
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(*#block))) }
                }
                SelfType::Owned if output_ident == "Rc" || output_ident == "Arc" => {
                    let ungenerified = {
                        // clone so we don't modify the original output
                        let mut output = output.clone();
                        output.path.segments.last_mut().unwrap().arguments = PathArguments::None;
                        output
                    };
                    quote! { <#output>::new(Self(faux::MaybeFaux::Real(
                        #ungenerified::try_unwrap(#block).ok().expect("faux: failed to grab value from reference counter because it was not unique.")
                    ))) }
                }
                _ => return Err(unhandled_self_return(output)),
            }
        };

        Ok(Some(wrapped))
    }
}

impl MockData {
    pub fn create_when(&self, name: &syn::Ident, real_fn: &syn::ExprPath) -> syn::ImplItemFn {
        let MockData {
            arg_types,
            self_ty,
            generics,
            output,
            ..
        } = self;

        let when_ident =
            syn::Ident::new(&format!("_when_{}", name), proc_macro2::Span::call_site());
        let extra_when_lts = arg_types
            .iter()
            .flat_map(|a| &a.lifetimes)
            .map(|lt| syn::GenericParam::Lifetime(syn::LifetimeParam::new(lt.clone())));

        let name_str = name.to_string();
        let mut when_generics = (*generics).clone();
        when_generics.params.extend(extra_when_lts);
        when_generics.params.extend(
            output
                .lifetimes
                .iter()
                .map(|lt| syn::GenericParam::Lifetime(syn::LifetimeParam::new(lt.clone()))),
        );
        let faux_lifetime = syn::GenericParam::Lifetime(syn::LifetimeParam {
            attrs: vec![],
            lifetime: syn::Lifetime::new("'_faux_mock_lifetime", Span::call_site()),
            colon_token: None,
            bounds: Punctuated::new(),
        });

        when_generics.params.push(faux_lifetime.clone());
        let (when_impl_generics, _, when_where_clause) = when_generics.split_for_impl();

        syn::parse_quote! {
            pub fn #when_ident #when_impl_generics(&#faux_lifetime mut self) -> faux::When<#faux_lifetime, #self_ty, (#(#arg_types),*), #output, faux::matcher::AnyInvocation> #when_where_clause {
                match &mut self.0 {
                    faux::MaybeFaux::Faux(_maybe_faux_faux) => faux::When::new(
                        #real_fn,
                        #name_str,
                        _maybe_faux_faux
                    ),
                    faux::MaybeFaux::Real(_) => panic!("not allowed to stub a real instance!"),
                }
            }
        }
    }
}

fn unhandled_self_return(spanned: impl Spanned) -> darling::Error {
    darling::Error::custom("faux: the return type refers to the mocked struct in a way that faux cannot handle. Split this function into an `impl` block not marked by #[faux::methods]. If you believe this is a mistake or it's a case that should be handled by faux please file an issue").with_span(&spanned)
}

fn contains_self(ty: &syn::Type, path: &syn::TypePath) -> bool {
    match ty {
        // end recursion
        syn::Type::Path(p) => {
            p == path
                || (p.qself.is_none() && p.path.is_ident("Self"))
                || path_args_contains_self(&p.path, path)
        }
        // recurse to inner type
        syn::Type::Array(arr) => contains_self(&arr.elem, path),
        syn::Type::Group(g) => contains_self(&g.elem, path),
        syn::Type::Paren(t) => contains_self(&t.elem, path),
        syn::Type::Ptr(p) => contains_self(&p.elem, path),
        syn::Type::Reference(p) => contains_self(&p.elem, path),
        syn::Type::Slice(s) => contains_self(&s.elem, path),
        // check deeper
        syn::Type::BareFn(barefn) => {
            return_contains_self(&barefn.output, path)
                || barefn.inputs.iter().any(|i| contains_self(&i.ty, path))
        }
        syn::Type::ImplTrait(it) => bounds_contains_self(it.bounds.iter(), path),
        syn::Type::TraitObject(t) => bounds_contains_self(t.bounds.iter(), path),
        syn::Type::Tuple(t) => t.elems.iter().any(|t| contains_self(t, path)),
        syn::Type::Infer(_) | syn::Type::Macro(_) | syn::Type::Never(_) => false,
        other => {
            let other = other.to_token_stream().to_string();
            other.contains("Self") || other.contains(&path.to_token_stream().to_string())
        }
    }
}

fn ang_generic_contains_self(
    args: &syn::AngleBracketedGenericArguments,
    path: &syn::TypePath,
) -> bool {
    args.args.iter().any(|a| match a {
        syn::GenericArgument::Lifetime(_)
        | syn::GenericArgument::Const(_)
        | syn::GenericArgument::AssocConst(_)
        | syn::GenericArgument::Constraint(_) => false,
        syn::GenericArgument::Type(ty) => contains_self(ty, path),
        syn::GenericArgument::AssocType(assoc) => {
            if contains_self(&assoc.ty, path) {
                return true;
            }
            // TODO: use let-else once we bump MSRV to 1.65.0
            let args = match &assoc.generics {
                Some(args) => args,
                None => return false,
            };
            ang_generic_contains_self(args, path)
        }
        other => {
            let other = other.to_token_stream().to_string();
            other.contains("Self") || other.contains(&path.to_token_stream().to_string())
        }
    })
}

fn return_contains_self(ret: &syn::ReturnType, path: &syn::TypePath) -> bool {
    match &ret {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => contains_self(ty, path),
    }
}

fn bounds_contains_self<'b>(
    mut bounds: impl Iterator<Item = &'b syn::TypeParamBound>,
    path: &syn::TypePath,
) -> bool {
    bounds.any(|b| match b {
        syn::TypeParamBound::Trait(t) => path_args_contains_self(&t.path, path),
        syn::TypeParamBound::Lifetime(_) => false,
        syn::TypeParamBound::Verbatim(ts) => {
            let ts = ts.to_token_stream().to_string();
            ts.contains("Self") || ts.contains(&path.to_token_stream().to_string())
        }
        other => {
            let other = other.to_token_stream().to_string();
            other.contains("Self") || other.contains(&path.to_token_stream().to_string())
        }
    })
}

fn path_args_contains_self(path: &syn::Path, self_path: &syn::TypePath) -> bool {
    match &path.segments.last().unwrap().arguments {
        PathArguments::None => false,
        PathArguments::AngleBracketed(args) => ang_generic_contains_self(args, self_path),
        PathArguments::Parenthesized(args) => {
            return_contains_self(&args.output, self_path)
                || args.inputs.iter().any(|i| contains_self(i, self_path))
        }
    }
}

fn implicit_ref_lifetime(ty: &mut syn::Type) -> Option<(syn::Lifetime, bool)> {
    let span = ty.span();
    let r = match ty {
        syn::Type::Reference(r) => r,
        _ => return None,
    };

    let lifetime = match &mut r.lifetime {
        Some(lt) => (lt.clone(), false),
        None => {
            let lifetime = syn::Lifetime::new("'_faux_implicit_ref", span);
            r.lifetime = Some(lifetime.clone());
            (lifetime, true)
        }
    };

    Some(lifetime)
}
