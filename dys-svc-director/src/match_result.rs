use dys_datastore_valkey::datastore::AsyncCommands;
use dys_nats::error::NatsError;
use dys_protocol::nats::match_results::{GetGameLogRequest, GetGameLogResponse, MatchRequest, MatchResponse};
use crate::AppState;

pub async fn get_summaries(
    request: MatchRequest,
    mut app_state: AppState
    // ZJ-TODO: the error type of the signature should be Result<Response, CustomError>, not NatsError
) -> Result<MatchResponse, NatsError> {
    let mut valkey = app_state.valkey.connection();

    let match_ids: Vec<u64> = {
        if request.match_ids.len() > 0 {
            request.match_ids
        } else {
            // If no match IDs are provided, just grab the latest ~10
            valkey.lrange(
                "env:dev:match.results:latest",
                0,
                9,
            ).await.unwrap()
        }
    };

    let mut match_summaries = Vec::new();
    for match_id in match_ids {
        let response_data: String = valkey.hget(
            format!("env:dev:match.results:id:{match_id}"),
            "summary",
        ).await.unwrap();

        match_summaries.push(serde_json::from_str(&response_data).unwrap());
    }

    let response = MatchResponse {
        match_summaries,
    };
    Ok(response)
}

pub async fn get_game_log(
    request: GetGameLogRequest,
    mut app_state: AppState,
) -> Result<GetGameLogResponse, NatsError> {
    let mut valkey = app_state.valkey.connection();
    let game_log_serialized: Vec<u8> = valkey.hget(
        format!("env:dev:match.results:id:{}", request.match_id),
        "game_log"
    ).await.unwrap();

    let response = GetGameLogResponse {
        game_log_serialized
    };

    Ok(response)
}