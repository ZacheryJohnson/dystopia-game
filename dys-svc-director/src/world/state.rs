// ZJ-TODO: this entire file should ideally be deprecated
//          requesting the entire world as a JSON isn't great,
//          especially if we're keeping multiple revisions

use utoipa::IntoParams;
use utoipa::openapi::path::Parameter;
use utoipa::openapi::path::ParameterIn;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use dys_datastore_valkey::datastore::AsyncCommands;
use dys_nats::error::NatsError;
use dys_service_base_macros::{api, ApiRequest};
use crate::AppState;

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "World State",
        description = "SOON TO BE DEPRECATED. World state as of the current simulation",
    ),
    paths(get_world_state),
)]
pub struct WorldStateApi;

#[derive(Debug, Clone, Serialize, Deserialize, ApiRequest)]
pub struct GetWorldStateRequest {
    revision: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetWorldStateResponse {
    pub world_state_json: String,
}

#[api(
    request = GetWorldStateRequest,
    response = GetWorldStateResponse,
    error = NatsError,
    app_state = AppState,
    http(
        method = "Get",
        path = "",
    ),
    nats(
        topic = "api.v1.world.state.get",
    ),
)]
pub async fn get_world_state(
    _: GetWorldStateRequest,
    app_state: AppState,
) -> Result<GetWorldStateResponse, NatsError> {
    // ZJ-TODO: read response for particular world revision

    let mut valkey = app_state.valkey.lock().unwrap().connection();
    let response_data: String = valkey.hget("env:dev:world", "data").await.unwrap();

    Ok(GetWorldStateResponse {
        world_state_json: response_data,
    })
}