use dys_datastore_valkey::datastore::AsyncCommands;
use dys_nats::error::NatsError;
use dys_protocol::nats::match_results::{MatchRequest, MatchResponse};
use crate::WorldState;

pub async fn get_summaries(
    _: MatchRequest,
    mut app_state: WorldState
) -> Result<MatchResponse, NatsError> {
    let mut valkey = app_state.valkey.connection();
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

    let response = MatchResponse {
        match_summaries,
    };
    Ok(response)
}
