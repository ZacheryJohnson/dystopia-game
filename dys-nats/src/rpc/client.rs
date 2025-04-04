use std::fmt::Debug;
use std::time::Duration;
use async_nats::HeaderMap;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::NatsError;
use crate::otel::propagate_otel_context;

pub trait NatsRpcClient {
    type Request: Serialize + DeserializeOwned + Debug;
    type Response: Serialize + DeserializeOwned + Debug;

    const RPC_SUBJECT: &'static str;

    fn client(&self) -> async_nats::Client;

    async fn send_request(
        &mut self,
        request: Self::Request,
    ) -> Result<Self::Response, NatsError> {
        let nats_client = self.client();
        let reply_subject = nats_client.new_inbox();
        let result = nats_client.subscribe(
            reply_subject.clone()
        ).await;

        let Ok(mut reply_subscriber) = result else {
            tracing::error!("failed to subscribe to reply topic {reply_subject}");
            return Err(NatsError::ReplySubjectSubscribeError);
        };

        let mut headers = HeaderMap::new();
        propagate_otel_context(&mut headers);

        let payload = postcard::to_allocvec(&request).unwrap();
        let result = nats_client.publish_with_reply_and_headers(
            Self::RPC_SUBJECT,
            reply_subject,
            headers,
            payload.into()
        ).await;

        if result.is_err() {
            tracing::error!("failed to publish summary request: {:?}", result.err().unwrap());
            return Err(NatsError::PublishError);
        }

        let Ok(response) = tokio::time::timeout(Duration::from_millis(10000), async {
            loop {
                let Some(response) = reply_subscriber.next().await else {
                    continue;
                };

                return response;
            }
        }).await else {
            tracing::error!("timed out waiting for reply");
            return Err(NatsError::PublishTimeout);
        };

        let response: Self::Response = postcard::from_bytes(&response.payload.to_vec()).unwrap();
        Ok(response)
    }
}