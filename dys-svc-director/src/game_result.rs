use std::collections::HashMap;
use dys_datastore_valkey::datastore::{AsyncCommands, AsyncIter};
use dys_nats::error::NatsError;
use dys_protocol::nats::game_results::{GameSummaryRequest, GameSummaryResponse, GetGameLogRequest, GetGameLogResponse};
use dys_protocol::nats::game_results::game_summary_response::GameSummary;
use crate::AppState;

#[tracing::instrument(skip_all)]
pub async fn get_summaries(
    request: GameSummaryRequest,
    app_state: AppState
    // ZJ-TODO: the error type of the signature should be Result<Response, CustomError>, not NatsError
) -> Result<GameSummaryResponse, NatsError> {
    let mut valkey = app_state.valkey.lock().unwrap().connection();

    let game_ids: Vec<u64> = {
        if !request.game_ids.is_empty() {
            request.game_ids
        } else {
            // If no game IDs are provided, just grab the latest ~10
            valkey.lrange(
                "env:dev:game.results:latest",
                0,
                9,
            ).await.unwrap()
        }
    };

    let mut game_summaries = Vec::new();
    for game_id in game_ids {
        let response_data: String = valkey.hget(
            format!("env:dev:game.results:id:{game_id}"),
            "summary",
        ).await.unwrap();

        game_summaries.push(serde_json::from_str(&response_data).unwrap());
    }

    // Get team records
    let mut team_records: HashMap<String, (u32, u32)> = HashMap::new();
    let mut keys = vec![];
    {
        let mut iter: AsyncIter<String> = valkey.scan_match("env:dev:season:record:team:*").await.unwrap();
        while let Some(record_key) = iter.next_item().await {
            keys.push(record_key);
        }
    }

    for maybe_key in keys {
        let Ok(key) = maybe_key else {
            tracing::warn!("failed to parse key: {:?}", maybe_key.err());
            continue;
        };

        // ZJ-TODO: pipeline this
        let raw_values: Vec<Option<u32>> = valkey.hget(
            key.clone(),
            vec!["wins", "losses"]
        ).await.unwrap();

        let wins = raw_values[0].unwrap_or(0);
        let losses = raw_values[1].unwrap_or(0);

        team_records.insert(key.rsplit(":").next().unwrap().to_string(), (wins, losses));
    }

    let mut next_games = Vec::new();
    {
        let season = app_state.season.lock().unwrap();
        let current_date = app_state.current_date.lock().unwrap();

        for game_instance in season.games_on_date(&current_date) {
            let game_instance = game_instance.upgrade().unwrap();
            let game_instance = game_instance.lock().unwrap();
            let home_team = game_instance.home_team.lock().unwrap();
            let away_team = game_instance.away_team.lock().unwrap();
            next_games.push(GameSummary {
                game_id: Some(game_instance.game_id),
                away_team_name: Some(away_team.name.clone()),
                home_team_name: Some(home_team.name.clone()),
                away_team_score: None,
                home_team_score: None,
                date: Some(dys_protocol::nats::common::Date {
                    year: current_date.2,
                    month: current_date.0.to_owned() as i32 + 1,
                    day: current_date.1,
                }),
                home_team_record: Some(format!(
                    "{}-{}",
                    team_records.get(&home_team.name).unwrap_or(&(0, 0)).0,
                    team_records.get(&home_team.name).unwrap_or(&(0, 0)).1,
                )),
                away_team_record: Some(format!(
                    "{}-{}",
                    team_records.get(&away_team.name).unwrap_or(&(0, 0)).0,
                    team_records.get(&away_team.name).unwrap_or(&(0, 0)).1,
                )),
            });
        }
    }

    let response = GameSummaryResponse {
        game_summaries,
        next_games,
    };
    Ok(response)
}

#[tracing::instrument(skip_all)]
pub async fn get_game_log(
    request: GetGameLogRequest,
    app_state: AppState,
) -> Result<GetGameLogResponse, NatsError> {
    let mut valkey = app_state.valkey.lock().unwrap().connection();
    let game_log_serialized: Vec<u8> = valkey.hget(
        format!("env:dev:game.results:id:{}", request.game_id.as_ref().unwrap_or(&0)),
        "game_log"
    ).await.unwrap();

    let response = GetGameLogResponse {
        game_log_serialized: Some(game_log_serialized),
    };

    Ok(response)
}