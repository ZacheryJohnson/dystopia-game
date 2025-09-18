use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::{parenthesized, parse_macro_input, FnArg, ItemFn, Pat, Token, Type};
use crate::http::{http_openapi_header_impl, HttpApiAttribute};
use crate::nats::{natsapi_impl, NatsApiAttribute};

#[derive(Default)]
struct ApiAttribute {
    /// The type of request the API expects.
    /// Required.
    request_type: Option<Type>,

    /// The type of response the API will provide if the request is handled successfully.
    /// Required.
    response_type: Option<Type>,

    /// The type of response the API will provide if the request is not handled successfully.
    /// Required.
    error_type: Option<Type>,

    /// The type of application state that will be passed alongside the request, such as database
    /// connections.
    /// Required.
    app_state_type: Option<Type>,

    /// At least one of \[`http_options`, `nats_options`] must be provided.
    http_options: Option<HttpApiAttribute>,

    /// At least one of \[`http_options`, `nats_options`] must be provided.
    nats_options: Option<NatsApiAttribute>,
}

impl syn::parse::Parse for ApiAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attributes = ApiAttribute::default();
        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                syn::Error::new(
                    error.span(),
                    format!("unexpected attribute: {error} (input: '{}')", input.to_string()),
                )
            })?;

            match &*ident.to_string().as_str() {
                "request" => {
                    input.parse::<Token![=]>()?;
                    attributes.request_type = Some(input.parse()?);
                },
                "response" => {
                    input.parse::<Token![=]>()?;
                    attributes.response_type = Some(input.parse()?);
                },
                "error" => {
                    input.parse::<Token![=]>()?;
                    attributes.error_type = Some(input.parse()?);
                },
                "app_state" => {
                    input.parse::<Token![=]>()?;
                    attributes.app_state_type = Some(input.parse()?);
                },
                "http" => {
                    let content;
                    parenthesized!(content in input);
                    let http_attributes: HttpApiAttribute = content.parse()?;
                    attributes.http_options = Some(http_attributes);
                },
                "nats" => {
                    let content;
                    parenthesized!(content in input);
                    let nats_attributes: NatsApiAttribute = content.parse()?;
                    attributes.nats_options = Some(nats_attributes);
                }
                _ => {
                    return Err(syn::Error::new(ident.span(), "unexpected attribute"));
                }
            }

            let _ = input.parse::<Token![,]>();
        }

        if attributes.request_type.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "`request` must be defined")
            );
        }

        if attributes.response_type.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "`response` must be defined")
            );
        }

        if attributes.error_type.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "`error` must be defined")
            );
        }

        if attributes.app_state_type.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "`app_state` must be defined")
            );
        }

        if attributes.http_options.is_none() && attributes.nats_options.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "at least one of `http` or `nats` must be defined")
            );
        }

        Ok(attributes)
    }
}

pub fn api_impl(attribute: proc_macro::TokenStream, api: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let api_attribute = parse_macro_input!(attribute as ApiAttribute);
    let api_fn = parse_macro_input!(api as ItemFn);

    let mut token_stream = proc_macro::TokenStream::new();

    // ZJ-TODO: check type of app_state vs api_attribute.app_state
    let app_state_var = {
        let mut app_state_ident = None;
        for param in &api_fn.sig.inputs {
            let FnArg::Typed(param) = param else {
                continue;
            };

            if *param.ty == *api_attribute.app_state_type.as_ref().unwrap() {
                let Pat::Ident(ident) = &*param.pat else {
                    panic!("expected identifier for type");
                };

                app_state_ident = Some(ident);
                break;
            }
        }

        app_state_ident
    };

    if let Some(nats_attributes) = api_attribute.nats_options {
        let nats_token_stream: proc_macro::TokenStream = natsapi_impl(
            nats_attributes,
            api_fn.sig.ident.to_string(),
            api_attribute.request_type.as_ref().unwrap().clone(),
            api_attribute.response_type.as_ref().unwrap().clone(),
        );

        token_stream.extend(nats_token_stream);
    }

    // Code below this point may only add new attributes to the function

    let tracing_macro_stream: proc_macro::TokenStream = quote! {
        #[tracing::instrument(skip(#app_state_var))]
    }.into();

    token_stream.extend(tracing_macro_stream);

    if let Some(http_attributes) = api_attribute.http_options {
        let openapi_macro_stream: proc_macro::TokenStream = http_openapi_header_impl(
            http_attributes,
            api_attribute.request_type.as_ref().unwrap().clone(),
            api_attribute.response_type.as_ref().unwrap().clone(),
        );

        token_stream.extend(openapi_macro_stream);
    }

    let fn_tokens: proc_macro::TokenStream = api_fn.to_token_stream().into();
    token_stream.extend(fn_tokens);

    // panic!("{token_stream}");

    token_stream
}