use std::collections::HashMap;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use dys_datastore_valkey::datastore::{AsyncCommands, AsyncIter};
use dys_nats::error::NatsError;
use dys_service_base_macros::{api, ApiRequest};
use crate::AppState;
use crate::game::types::{GamePreview, GameSummary};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(get_summaries)
)]
pub struct SummaryApi;

#[derive(Debug, Clone, Serialize, Deserialize, ApiRequest)]
pub struct GetGameSummaryRequest {

}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetGameSummaryResponse {
    pub game_summaries: Vec<GameSummary>,
    pub next_games: Vec<GamePreview>,
}

#[api(
    request = GetGameSummaryRequest,
    app_state = AppState,
    http(
        method = "Get",
        path = "",
    ),
    nats(
        topic = "api.v1.game.summary.get",
    ),
)]
pub async fn get_summaries(
    request: GetGameSummaryRequest,
    app_state: AppState
    // ZJ-TODO: the error type of the signature should be Result<Response, CustomError>, not NatsError
) -> Result<GetGameSummaryResponse, NatsError> {
    let mut valkey = app_state.valkey.lock().unwrap().connection();

    let game_ids: Vec<u32> = valkey.lrange(
        "env:dev:game.results:latest",
        0,
        9,
    ).await.unwrap();

    let mut game_summaries = Vec::new();
    for game_id in game_ids {
        let response_data: String = valkey.hget(
            format!("env:dev:game.results:id:{game_id}"),
            "summary",
        ).await.unwrap();

        tracing::info!("{response_data}");

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

        team_records.insert(key.rsplit(':').next().unwrap().to_string(), (wins, losses));
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
            next_games.push(GamePreview {
                game_id: game_instance.game_id,
                away_team_name: away_team.name.clone(),
                home_team_name: home_team.name.clone(),
                date: current_date.as_iso8601(),
                home_team_record: format!(
                    "{}-{}",
                    team_records.get(&home_team.name).unwrap_or(&(0, 0)).0,
                    team_records.get(&home_team.name).unwrap_or(&(0, 0)).1,
                ),
                away_team_record: format!(
                    "{}-{}",
                    team_records.get(&away_team.name).unwrap_or(&(0, 0)).0,
                    team_records.get(&away_team.name).unwrap_or(&(0, 0)).1,
                ),
            });
        }
    }

    let response = GetGameSummaryResponse {
        game_summaries,
        next_games,
    };
    Ok(response)
}