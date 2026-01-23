use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use dys_datastore_valkey::datastore::{AsyncCommands};
use dys_nats::error::NatsError;
use dys_service_base_macros::{api, ApiRequest};
use dys_world::games::instance::GameInstanceId;
use crate::AppState;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(get_game_log)
)]
pub struct LogApi;

#[derive(Clone, Debug, Serialize, Deserialize, ApiRequest)]
pub struct GetGameLogRequest {
    game_id: GameInstanceId,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct GetGameLogResponse {
    game_log_serialized: Vec<u8>,
}

#[api(
    request = GetGameLogRequest,
    app_state = AppState,
    http(
        method = "Get",
        path = "/{game_id}",
    ),
    nats(
        topic = "api.v1.game.log.by_game_id.get",
    ),
)]
pub async fn get_game_log(
    request: GetGameLogRequest,
    app_state: AppState,
) -> Result<GetGameLogResponse, NatsError> {
    tracing::info!("getting game log!");
    let mut valkey = app_state.valkey.lock().unwrap().connection();
    let game_log_serialized: Vec<u8> = valkey.hget(
        format!("env:dev:game.results:id:{}", request.game_id),
        "game_log"
    ).await.unwrap();

    let response = GetGameLogResponse {
        game_log_serialized
    };

    Ok(response)
}