use dys_datastore_valkey::datastore::AsyncCommands;
use dys_nats::error::NatsError;
use dys_protocol::nats::match_results::{GetGameLogRequest, GetGameLogResponse, MatchRequest, MatchResponse};
use dys_protocol::nats::match_results::match_response::MatchSummary;
use crate::AppState;

#[tracing::instrument(skip_all)]
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

    let mut next_matches = Vec::new();
    {
        let season = app_state.season.lock().unwrap();
        let current_date = app_state.current_date.lock().unwrap();

        for match_instance in season.matches_on_date(&*current_date) {
            let match_instance = match_instance.lock().unwrap();
            next_matches.push(MatchSummary {
                match_id: match_instance.match_id,
                away_team_name: match_instance.away_team.lock().unwrap().name.clone(),
                home_team_name: match_instance.home_team.lock().unwrap().name.clone(),
                away_team_score: 0,
                home_team_score: 0,
                date: Some(dys_protocol::nats::common::Date {
                    year: current_date.2,
                    month: current_date.0.to_owned() as i32 + 1,
                    day: current_date.1,
                }),
                home_team_record: "ZJ-TODO".to_string(),
                away_team_record: "ZJ-TODO".to_string(),
            });
        }
    }

    let response = MatchResponse {
        match_summaries,
        next_matches
    };
    Ok(response)
}

#[tracing::instrument(skip_all)]
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