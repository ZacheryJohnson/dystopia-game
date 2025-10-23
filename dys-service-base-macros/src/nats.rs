use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::ParseStream;
use syn::{ Lit, Token, Type};

#[derive(Default)]
pub(crate) struct NatsApiAttribute {
    pub topic: Option<String>,
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

pub fn natsapi_impl(
    nats_api_attribute: NatsApiAttribute,
    api_name: String,
    request_type: Type,
    response_type: Type,
) -> proc_macro::TokenStream {
    let struct_name = snake_case_to_pascal_case(api_name.to_string());
    // ZJ-TODO: don't have a NATS API attribute at all; just parse it from passed args from parent
    //          a la "api.vX.{endpoint_path}"
    let topic = nats_api_attribute.topic.unwrap();
    let service_struct_name = format_ident!("{}NatsService", struct_name);

    let fn_ident = format_ident!("{}", api_name);

    quote! {
        pub mod nats {
            use std::pin::Pin;
            use std::task::{Context, Poll};
            use bytes::Bytes;
            use futures::future::BoxFuture;
            use super::AppState;

            pub struct #service_struct_name {
                pub topic: String,
                app_state: AppState,
                handler_fn: Box<dyn Fn(super::#request_type, AppState) -> BoxFuture<'static, Result<super::#response_type, crate::NatsError>> + Send>,
            }

            impl #service_struct_name {
                pub fn from(app_state: AppState) -> #service_struct_name {
                    #service_struct_name {
                        topic: #topic.to_string(),
                        app_state,
                        handler_fn: Box::new(move |arg, app_state| Box::pin(super::#fn_ident(arg, app_state)))
                    }
                }
            }

            impl tower::Service<async_nats::Message> for #service_struct_name {
                type Response = Bytes;
                type Error = crate::NatsError; // ZJ-TODO
                type Future = Pin<Box<dyn Future<Output = Result<Bytes, Self::Error>> + Send>>;

                fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                    Poll::Ready(Ok(()))
                }

                fn call(&mut self, req: async_nats::Message) -> Self::Future {
                    let payload = req.payload.to_vec();
                    let Ok(converted_request) = serde_json::from_slice(&payload) else {
                        return Box::pin(async move {
                            Err(crate::NatsError::MalformedRequest)
                        });
                    };

                    let future = (self.handler_fn)(converted_request, self.app_state.clone());
                    Box::pin(async move {
                        let response = future.await;
                        match response {
                            Ok(resp) => Ok(serde_json::to_string(&resp).unwrap().into()),
                            Err(err) => Err(err),
                        }
                    })
                }
            }
        }
    }.into()
}