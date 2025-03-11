use std::pin::Pin;
use async_nats::{HeaderMap, Subscriber};
use bytes::Bytes;
use futures::stream::StreamExt;
use crate::error::NatsError;

struct NatsService {
    service: Box<(dyn tower::Service<
            async_nats::Message,
            Error=NatsError,
            Future=Pin<Box<dyn Future<Output = Result<Bytes, NatsError>> + Send>>,
            Response=Bytes,
        > + 'static
    )>,
    reply_inbox: String,
    subscription: Option<Subscriber>,
}

impl NatsService {
    async fn dispatch_message(
        &mut self,
        message: async_nats::Message
    ) -> Result<Bytes, NatsError> {
        self.service.call(message).await
    }
}

pub struct NatsRouter {
    client: async_nats::Client,
    services: Vec<NatsService>,
}

impl NatsRouter {
    pub async fn new() -> NatsRouter {
        let nats_server_str = format!(
            "{}:{}",
            std::env::var("NATS_HOST").unwrap_or(String::from("172.18.0.1")),
            std::env::var("NATS_PORT").unwrap_or(String::from("4222")).parse::<u16>().unwrap(),
        );

        let client = async_nats::ConnectOptions::new()
            .token(std::env::var("NATS_TOKEN").unwrap_or(String::from("replaceme")))
            .connect(nats_server_str)
            .await
            .unwrap();

        NatsRouter { client, services: Vec::new() }
    }

    pub fn service(
        mut self,
        service: (
            impl tower::Service<
                async_nats::Message,
                Error=NatsError,
                Future=Pin<Box<dyn Future<Output = Result<Bytes, NatsError>> + Send>>,
                Response=Bytes,
            > + 'static
        ),
    ) -> NatsRouter {
        let nats_service = NatsService {
            service: Box::new(service),
            reply_inbox: self.client.new_inbox(),
            subscription: None,
        };

        self.services.push(nats_service);
        self
    }

    pub async fn run(mut self) -> ! {
        for service in &mut self.services {
            let subscriber = self.client.subscribe(service.reply_inbox.to_owned()).await.unwrap();
            service.subscription = Some(subscriber);
        }

        loop {
            for service in &mut self.services {
                while let Some(message) = service.subscription.as_mut().unwrap().next().await {
                    let reply_subject = message.reply.as_ref().unwrap().to_owned();
                    let (response_payload, headers) = {
                        match service.dispatch_message(message).await {
                            Ok(response_payload) => (response_payload, HeaderMap::new()),
                            Err(err) => {
                                let mut headers = HeaderMap::new();
                                headers.insert("error", "true");
                                (postcard::to_allocvec(&err).unwrap().into(), headers)
                            },
                        }
                    };
                    self.client.publish_with_headers(
                        reply_subject,
                        headers,
                        response_payload,
                    ).await.unwrap();
                }
            }
        }
    }
}