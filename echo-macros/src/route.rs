use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Attribute, AttributeArgs, Error, Ident, ItemFn, Lit, LitStr, NestedMeta, Visibility};

pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);

    let item_fn = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(item_fn) => item_fn,
        Err(err) => return input_and_compile_error(input, err),
    };

    match Route::new(args, item_fn) {
        Ok(route) => route.into_token_stream().into(),
        Err(err) => input_and_compile_error(input, err),
    }
}

pub struct Args {
    path: LitStr,
    methods: HashSet<Method>,
}

impl Args {
    pub fn new(args: AttributeArgs) -> syn::Result<Self> {
        let mut path = None;
        let mut methods = HashSet::new();

        for arg in args {
            match arg {
                NestedMeta::Lit(Lit::Str(lit)) => match path {
                    None => {
                        path = Some(lit);
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            lit,
                            "Multiple paths specified! Should be only one!",
                        ));
                    }
                },
                NestedMeta::Meta(syn::Meta::NameValue(nv)) => {
                    if let Lit::Str(ref lit) = nv.lit {
                        if !methods.insert(Method::try_from(lit)?) {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                &format!("HTTP method defined more than once: `{}`", lit.value()),
                            ));
                        }
                    } else {
                        return Err(Error::new_spanned(
                            nv.lit,
                            "Attribute method expects literal string!",
                        ));
                    }
                }
                arg => {
                    return Err(Error::new_spanned(arg, "Unknown attribute."));
                }
            }
        }

        Ok(Self {
            path: path.ok_or_else(|| {
                Error::new(
                    Span::call_site(),
                    format!(r#"invalid route definition, expected #[route("<path>")]"#,),
                )
            })?,
            methods,
        })
    }
}

pub struct Route {
    item_fn: ItemFn,
    vis: Visibility,
    name: Ident,
    args: Args,
    docs: Vec<Attribute>,
}

impl Route {
    pub fn new(args: AttributeArgs, item_fn: ItemFn) -> syn::Result<Self> {
        let vis = item_fn.vis.clone();
        let name = item_fn.sig.ident.clone();

        let docs = item_fn
            .attrs
            .iter()
            .filter(|attr| attr.path.is_ident("doc"))
            .cloned()
            .collect();

        let args = Args::new(args)?;

        Ok(Self {
            item_fn,
            vis,
            name,
            args,
            docs,
        })
    }
}

impl ToTokens for Route {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            item_fn,
            vis,
            name,
            args,
            docs,
        } = self;

        let Args { path, methods } = args;

        let methods = methods.iter();

        let service = quote! {{
            ::echo::service::service_fn(#name)
        }};
        let arc_service = quote! {{
            let service = ::echo::service::ServiceExt::map_ok(#service, ::echo::response::IntoResponse::into_response);
            let service = ::echo::service::ServiceExt::map_err(service, ::std::convert::Into::into);
            <::echo::service::MapErr<_, _> as ::echo::service::ServiceExt<_>>::boxed_arc(service)
        }};
        let route = quote! {{
            let service = ::echo::route::any(#arc_service)
                #(.add(::echo::http::Method::try_from(#methods).unwrap()))*;
            ::echo::route::Route::new(#path, service)
        }};
        let arc_service_ty = quote! {
            ::echo::service::ArcService<::echo::Request, ::echo::Response, ::echo::BoxError>
        };
        let route_ty = quote! {
            ::echo::route::Route<#arc_service_ty>
        };

        let stream = quote! {
            #(#docs)*
            #[allow(non_camel_case_types)]
            #vis struct #name;

            impl #name {
                pub fn with<T>(self, middleware: T) -> ::echo::route::Route<T::Service>
                where
                    T: ::echo::middleware::Middleware<#arc_service_ty>,
                {
                    ::std::convert::Into::<#route_ty>::into(self).with(middleware)
                }
            }

            impl ::std::convert::Into<#route_ty> for #name {
                fn into(self) -> #route_ty {
                    #item_fn
                    #route
                }
            }
        };

        tokens.extend(stream);
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Method(String);

impl ToTokens for Method {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        stream.append(LitStr::new(self.0.as_str(), Span::call_site()).token());
    }
}

impl TryFrom<&LitStr> for Method {
    type Error = Error;

    fn try_from(value: &LitStr) -> Result<Self, Self::Error> {
        let method = value.value();
        if method.len() == 0 {
            Err(Error::new_spanned(value, "invalid HTTP method"))
        } else {
            Ok(Method(method))
        }
    }
}

/// Converts the error to a token stream and appends it to the original input.
///
/// Returning the original input in addition to the error is good for IDEs which can gracefully
/// recover and show more precise errors within the macro body.
///
/// See <https://github.com/rust-analyzer/rust-analyzer/issues/10468> for more info.
fn input_and_compile_error(mut item: TokenStream, err: Error) -> TokenStream {
    let compile_err = TokenStream::from(err.to_compile_error());
    item.extend(compile_err);
    item
}
