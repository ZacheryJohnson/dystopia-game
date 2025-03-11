use async_nats::Message;
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use futures::StreamExt;
use dys_datastore_valkey::datastore::{AsyncCommands, ValkeyDatastore};
use dys_protocol::http::match_results::match_response::MatchSummary;
use dys_protocol::http::match_results::MatchResponse;

pub struct SummaryService {
    valkey: ValkeyDatastore,
    nats_client: async_nats::Client,
    subscriber: Option<async_nats::Subscriber>,
}

impl SummaryService {
    pub fn new(
        valkey: ValkeyDatastore,
        nats_client: async_nats::Client
    ) -> Self {
        Self {
            valkey,
            nats_client,
            subscriber: None,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn initialize(&mut self) {
        // ZJ-TODO: abort early if we already have a subscriber
        if self.subscriber.is_some() {
            tracing::warn!("Already have a NATS subscriber!");
        }

        let subject = format!(
            "rpc.{}",
            dys_protocol::http::match_results::summary_server::SERVICE_NAME
        );
        let subscriber = self
            .nats_client
            .subscribe(subject)
            .await
            .expect("failed to subscribe to subject"); // ZJ-TODO: exponential backoff

        self.subscriber = Some(subscriber);
    }

    #[tracing::instrument(skip(self))]
    pub async fn process(&mut self) {
        if self.subscriber.is_none() {
            return;
        }

        let subscriber = self.subscriber.as_mut().unwrap();
        while let Some(request) = subscriber.next().await {
            if request.reply.is_none() {
                tracing::warn!("expected to receive a request with a reply subject");
                return;
            }

            let mut valkey = self.valkey.connection();
            let match_id_str: String = valkey.get("env:dev:match.results:latest").await.unwrap();
            let match_ids: Vec<u64> = serde_json::from_str(match_id_str.as_str()).unwrap();

            let mut match_summaries = Vec::new();
            for match_id in match_ids {
                let response_data: String = valkey.hget(
                    format!("env:dev:match.results:id:{match_id}"),
                    "data",
                ).await.unwrap();

                match_summaries.push(serde_json::from_str(&response_data).unwrap());
            }

            let match_response = MatchResponse {
                match_summaries,
            };

            let payload = postcard::to_allocvec(&match_response).unwrap();

            let result = self
                .nats_client
                .publish(request.reply.unwrap(), payload.into())
                .await;

            if let Err(e) = result {
                tracing::warn!("failed to publish response: {e}");
            }
        }
    }
}
