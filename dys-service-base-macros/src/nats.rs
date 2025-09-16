use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::ParseStream;
use syn::{ItemFn, Lit, Token, Type};

/// #[natsapi("api.stats.v1.recent", request = CombatantInstanceId, response = GetGameStatlinesResponse)]
#[derive(Default)]
pub(crate) struct NatsApiAttribute {
    pub topic: Option<String>,
    pub request_type: Option<Type>,
    pub response_type: Option<Type>,
}

impl syn::parse::Parse for NatsApiAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attributes = NatsApiAttribute::default();
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
                "topic" => {
                    input.parse::<Token![=]>()?;
                    let Lit::Str(parsed_topic) = input.parse::<Lit>()? else {
                        return Err(syn::Error::new(ident.span(), "expected string literal for topic"))
                    };

                    attributes.topic = Some(parsed_topic.value());
                },
                _ => return Err(syn::Error::new(ident.span(), "unexpected attribute")),
            }

            let _ = input.parse::<Token![,]>();
        }

        Ok(attributes)
    }
}

fn snake_case_to_pascal_case(input: String) -> String {
    input
        .replace("_", " ")
        .split(' ')
        .map(|part| part[0..1].to_uppercase() + &part[1..])
        .reduce(|acc, next| acc + &next)
        .unwrap_or_default()
}

pub fn natsapi_impl(attribute: proc_macro::TokenStream, api: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let nats_api_attribute = syn::parse_macro_input!(attribute as NatsApiAttribute);
    let api_fn = syn::parse_macro_input!(api as ItemFn);
    let api_name = &api_fn.sig.ident;
    let struct_name = snake_case_to_pascal_case(api_name.to_string());

    let topic = nats_api_attribute.topic.unwrap();
    let request_type = match nats_api_attribute.request_type {
        Some(request_type) => quote! { #request_type },
        None => quote! { () },
    };
    let response_type = match nats_api_attribute.response_type {
        Some(response_type) => quote! { #response_type },
        None => quote! { () },
    };

    let service_struct_name = format_ident!("{}NatsService", struct_name);

    let mut generated_tower_service = quote! {
        use std::pin::Pin;
        use std::task::{Context, Poll};
        use futures::future::BoxFuture;

        struct #service_struct_name {
            topic: String,
            app_state: AppState,
            handler_fn: Box<dyn Fn(#request_type, AppState) -> BoxFuture<'static, Result<#response_type, crate::NatsError>>>,
        }

        impl #service_struct_name {
            pub fn from(app_state: AppState) -> #service_struct_name {
                #service_struct_name {
                    topic: #topic.to_string(),
                    app_state,
                    handler_fn: Box::new(move |arg, app_state| Box::pin(#api_name(arg, app_state)))
                }
            }
        }

        impl tower::Service<async_nats::Message> for #service_struct_name {
            type Response = #response_type;
            type Error = crate::NatsError; // ZJ-TODO
            type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, req: async_nats::Message) -> Self::Future {
                let Ok(converted_request) = postcard::from_bytes(&req.payload.to_vec()) else {
                    return Box::pin(async { Err(dys_nats::error::NatsError::MalformedRequest) });
                };

                let future = (self.handler_fn)(converted_request, self.app_state.clone());
                Box::pin(async move {
                    future.await
                })
            }
        }
    };

    generated_tower_service.extend(api_fn.to_token_stream());
    generated_tower_service.into()
}