use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::Bytes;
use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use dys_nats::error::NatsError;

pub struct NatsEndpointHandler<Req, AppState> {
    pub nats_topic: String,

    app_state: AppState,
    handler_fn: Box<dyn Fn(Req, AppState) -> BoxFuture<'static, Result<Bytes, NatsError>> + Send>,
}

impl<Req, AppState> NatsEndpointHandler<Req, AppState> {
    pub fn from(
        topic: impl Into<String>,
        app_state: AppState,
        handler: Box<dyn Fn(Req, AppState) -> BoxFuture<'static, Result<Bytes, NatsError>> + Send>
    ) -> Self {
        NatsEndpointHandler {
            nats_topic: topic.into(),
            app_state,
            handler_fn: handler,
        }
    }
}

impl<Req: DeserializeOwned, AppState: Clone> tower::Service<async_nats::Message> for NatsEndpointHandler<Req, AppState> {
    type Response = Bytes;
    type Error = NatsError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: async_nats::Message) -> Self::Future {
        let payload = req.payload.to_vec();
        let Ok(converted_request) = serde_json::from_slice(&payload) else {
            return Box::pin(async move {
                Err(NatsError::MalformedRequest)
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