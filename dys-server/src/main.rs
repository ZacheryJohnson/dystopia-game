use std::time::Duration;

use director_api::director_service_server::{DirectorService, DirectorServiceServer};
use director_api::{CreateNewDirectorRequest, CreateNewDirectorResponse};

use dys_director::director::Director;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[path = "../generated/director.rs"]
pub mod director_api;

#[derive(Default)]
pub struct DysDirectorService;

#[tonic::async_trait]
impl DirectorService for DysDirectorService {
    async fn create_new_director(
        &self,
        _request: Request<CreateNewDirectorRequest>,
    ) -> Result<Response<CreateNewDirectorResponse>, Status> {
        let director = Director::new();
        tokio::spawn(async move {
            loop {
                director.tick();
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        Ok(Response::new(CreateNewDirectorResponse::default()))
    }
}

fn register_tracing_subscriber() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "dystopia".into(), 
        std::io::stdout
    );
    
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    
    set_global_default(subscriber).expect("Failed to set subscriber");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    register_tracing_subscriber();

    let addr = "127.0.0.1:3000".parse().unwrap();
    let dys_director_svc = DysDirectorService::default();

    println!("Starting DysDirectorService on {}", addr);

    Server::builder()
        .add_service(DirectorServiceServer::new(dys_director_svc))
        .serve(addr)
        .await?;
    
    Ok(())
}