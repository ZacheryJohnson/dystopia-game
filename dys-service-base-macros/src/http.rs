use std::fmt::Display;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::ParseStream;
use syn::{Lit, Token, Type};

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,

    // Other methods unsupported by this crate
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Default)]
pub(crate) struct HttpApiAttribute {
    pub path: Option<String>,

    pub method: Option<HttpMethod>,
}

impl syn::parse::Parse for HttpApiAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attributes = HttpApiAttribute::default();
        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                syn::Error::new(
                    error.span(),
                    format!("unexpected attribute: {error} (input: '{}')", input.to_string()),
                )
            })?;

            match &*ident.to_string().as_str() {
                "path" => {
                    input.parse::<Token![=]>()?;
                    let Lit::Str(parsed_path) = input.parse::<Lit>()? else {
                        return Err(syn::Error::new(
                            ident.span(),
                            "expected string literal for path"
                        ));
                    };

                    attributes.path = Some(parsed_path.value());
                },
                "method" => {
                    input.parse::<Token![=]>()?;
                    let Lit::Str(parsed_method) = input.parse::<Lit>()? else {
                        return Err(syn::Error::new(
                            ident.span(),
                            "expected string literal for method"
                        ));
                    };

                    let method_str = parsed_method.value();
                    match method_str.to_ascii_lowercase().as_str() {
                        "get" => attributes.method = Some(HttpMethod::Get),
                        "post" => attributes.method = Some(HttpMethod::Post),
                        "put" => attributes.method = Some(HttpMethod::Put),
                        "patch" => attributes.method = Some(HttpMethod::Patch),
                        "delete" => attributes.method = Some(HttpMethod::Delete),
                        _ => return Err(syn::Error::new(
                            ident.span(),
                            "expected string literal for method"
                        )),
                    }
                }
                _ => return Err(syn::Error::new(ident.span(), "unexpected attribute")),
            }

            let _ = input.parse::<Token![,]>();
        }

        Ok(attributes)
    }
}

pub fn http_openapi_header_impl(
    http_attribute: HttpApiAttribute,
    request_type: Type,
    response_type: Type,
) -> proc_macro::TokenStream {
    let method_type = format_ident!("{}", http_attribute.method.as_ref().unwrap().to_string().to_ascii_lowercase());
    let path = http_attribute.path.unwrap();

    // ZJ-TODO: security
    // ZJ-TODO: responses

    // ZJ-TODO: blindly assuming non-GET methods won't have params, which is wrong
    match &http_attribute.method.unwrap() {
        HttpMethod::Get => {
            quote! {
                #[utoipa::path(
                    method(#method_type),
                    path = #path,
                    params(
                        #request_type,
                    ),
                    responses(
                        (status = 200, body = #response_type),
                    ),
                )]
            }.into()
        }
        _ => {
            quote! {
                #[utoipa::path(
                    method(#method_type),
                    path = #path,
                    request_body = #request_type,
                    responses(
                        (status = 200, body = #response_type),
                    ),
                )]
            }.into()
        }
    }
}