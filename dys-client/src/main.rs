#[path = "../generated/director.rs"]
pub mod director_api;
use director_api::CreateNewDirectorRequest;

use crate::director_api::director_service_client::DirectorServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DirectorServiceClient::connect("http://127.0.0.1:3000").await?;

    let request = tonic::Request::new(CreateNewDirectorRequest {

    });

    let response = client.create_new_director(request).await?;

    tracing::info!("Response: {}", response.get_ref().director_id);

    Ok(())
}