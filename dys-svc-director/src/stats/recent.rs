use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::IntoParams;
use utoipa::openapi::path::{Parameter, ParameterIn};
use dys_datastore_mysql::fetch_all_query;
use dys_datastore_mysql::query::MySqlQuery;
use dys_nats::error::NatsError;
use dys_protocol::nats::stats::get_game_statlines_response::GameStatlines;
use dys_protocol::nats::stats::GetGameStatlinesResponse;
use dys_world::combatant::instance::CombatantInstanceId;
use dys_service_base_macros::{api, natsapi, ApiRequest};
use crate::AppState;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_recent_stats))]
pub struct RecentApi;

#[derive(Debug)]
struct GetRecentGamesQuery {
    combatant_id: CombatantInstanceId,
    games_count: u32,
}

impl MySqlQuery for GetRecentGamesQuery {
    fn query(&mut self) -> impl sqlx::Execute<'_, sqlx::MySql> {
        sqlx::query!("
            SELECT
                game_id,
                points,
                balls_thrown,
                throws_hit,
                combatants_shoved
            FROM game_statline
            WHERE combatant_id = ?
            ORDER BY game_id DESC -- ZJ-TODO: this should be by game date, not game id
            LIMIT ?
        ",
            self.combatant_id,
            self.games_count
        )
    }
}

#[derive(Debug, Serialize, Deserialize, ApiRequest)]
struct GetRecentStatsRequest {
    combatant_id: CombatantInstanceId,
    games_count: Option<u32>,
}

/*
    ZJ-TODO: ideal macro annotation
    #[derive(Serialize, Deserialize)]
    struct GetRecentStatsRequest {
        combatant_id: CombatantInstanceId,
        games_count: Option<u32>,
    }

    #[api(
        request = GetRecentStatsRequest,
        response = GetGameStatlinesResponse,
        error = ErrorType,
        http(
            path = "/{combatant_id}",  // <- pulled from request.combatant_id
            method = Get,
        ), // automatically generates the utopia::path params() from all possible request fields
        nats(
            topic = "api.stats.v1.recent"
        ),
     )]
     pub fn get_recent_stats(request: GetRecentStatsRequest, app_state: AppState) -> Result<GetGameStatlinesResponse, ErrorType> { ... }
 */

#[natsapi(
    topic = "api.stats.v1.recent",
    request = GetRecentStatsRequest,
    response = GetGameStatlinesResponse
)]
#[api(
    request = GetRecentStatsRequest,
    response = GetGameStatlinesResponse,
    error = (),
    app_state = AppState,
    http(
        method = "GET",
        path = "/{combatant_id}",
    ),
    nats(
        topic = "api.stats.v1.recent",
    ),
)]
pub async fn get_recent_stats(
    request: GetRecentStatsRequest,
    app_state: AppState
    // ZJ-TODO: the error type of the signature should be Result<Response, CustomError>, not NatsError
) -> Result<GetGameStatlinesResponse, NatsError> {
    const DEFAULT_NUMBER_OF_GAMES: u32 = 3;
    let games = fetch_all_query!(app_state.mysql.clone(), GetRecentGamesQuery {
        combatant_id: request.combatant_id,
        games_count: request.games_count.unwrap_or(DEFAULT_NUMBER_OF_GAMES),
    });

    let mut response = GetGameStatlinesResponse::default();
    for row in games {
        let game_id: u32 = row.get("game_id");
        let points: i64 = row.get("points");
        let balls_thrown: u64 = row.get("balls_thrown");
        let throws_hit: u64 = row.get("throws_hit");
        let combatants_shoved: u64 = row.get("combatants_shoved");

        #[derive(Serialize)]
        struct Statline {
            points: i64,
            throws: u64,
            hits: u64,
            shoves: u64,
        }
        let statline = Statline {
            points,
            throws: balls_thrown,
            hits: throws_hit,
            shoves: combatants_shoved
        };

        response.statlines.push(GameStatlines {
            game_id,
            combatant_statlines: HashMap::from([
                (request.combatant_id, serde_json::to_vec(&statline).unwrap())
            ]),
        });
    }

    Ok(response)
}