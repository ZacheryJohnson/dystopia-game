use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use async_nats::HeaderMap;
use bytes::Bytes;
use futures::stream::StreamExt;
use crate::error::NatsError;
use crate::server::NatsRpcServer;

struct NatsServiceInstance {
    service: Box<(dyn tower::Service<
            async_nats::Message,
            Error=NatsError,
            Future=Pin<Box<dyn Future<Output = Result<Bytes, NatsError>> + Send>>,
            Response=Bytes,
        > + Send + 'static)
    >,
    rpc_subject: String,
}

impl NatsServiceInstance {
    async fn dispatch_message(
        &mut self,
        message: async_nats::Message
    ) -> Result<Bytes, NatsError> {
        self.service.call(message).await
    }
}

pub struct NatsRouter {
    client: async_nats::Client,
    services: Vec<NatsServiceInstance>,
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
            impl NatsRpcServer + tower::Service<
                async_nats::Message,
                Error=NatsError,
                Future=Pin<Box<dyn Future<Output = Result<Bytes, NatsError>> + Send>>,
                Response=Bytes,
            > + Send + 'static
        ),
    ) -> NatsRouter {
        let rpc_subject = NatsRpcServer::rpc_subject(&service).to_string();
        let nats_service = NatsServiceInstance {
            service: Box::new(service),
            rpc_subject,
        };

        self.services.push(nats_service);
        self
    }

    pub async fn run(mut self) -> ! {
        let services = std::mem::replace(&mut self.services, Vec::new());
        let shutdown_signal = Arc::new(Mutex::new((false, false)));

        let mut thread_handles = vec![];
        for mut service in services {
            tracing::info!(
                "Subscribing to subject {}",
                service.rpc_subject
            );

            let mut subscriber = self.client.subscribe(service.rpc_subject.to_owned()).await.unwrap();
            let nats_client = self.client.to_owned();
            let shutdown_signal = shutdown_signal.clone();
            thread_handles.push(tokio::spawn(async move {
                let shutdown_signal = shutdown_signal.clone();
                loop {
                    let (should_shut_down, has_begun_shutdown) = {
                        *shutdown_signal.clone().lock().unwrap()
                    };

                    if should_shut_down && !has_begun_shutdown {
                        tracing::info!("Beginning drain of {}", service.rpc_subject);
                        nats_client.drain().await.unwrap();
                        *shutdown_signal.clone().lock().unwrap() = (true, true);
                    }

                    // If we go some duration without new messages, look for shutdown signals
                    while let Ok(Some(message)) = tokio::time::timeout(Duration::from_millis(10), subscriber.next()).await {
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

                        nats_client.publish_with_headers(
                            reply_subject,
                            headers,
                            response_payload,
                        ).await.unwrap();
                    }

                    if should_shut_down {
                        tracing::info!("Drain of {} complete!", service.rpc_subject);
                        return;
                    }
                }
            }));
        }

        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Shutdown signal received, beginning shut down...");
                *shutdown_signal.clone().lock().unwrap() = (true, false);
            },
            _ = terminate => {
                tracing::info!("Shutdown signal received, beginning shut down...");
                *shutdown_signal.clone().lock().unwrap() = (true, false);
            },
        }

        loop {
            thread_handles.retain(|thread| !thread.is_finished());
            if thread_handles.is_empty() {
                tracing::info!("All services have stopped, shutting down...");
                std::process::exit(0);
            }
        }
    }
}