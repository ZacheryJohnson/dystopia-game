use std::collections::HashMap;
use sqlx::{Execute, MySql, Row};
use dys_datastore_mysql::fetch_all_query;
use dys_datastore_mysql::query::MySqlQuery;
use dys_nats::error::NatsError;
use dys_protocol::nats::stats::*;
use dys_world::combatant::instance::CombatantInstanceId;
use serde::Serialize;
use dys_protocol::nats::stats::get_game_statlines_response::GameStatlines;
use crate::AppState;

#[derive(Debug)]
struct GetSeasonTotalsQuery {
    season_id: u32,
}

impl MySqlQuery for GetSeasonTotalsQuery {
    fn query(&mut self) -> impl Execute<MySql> {
        sqlx::query!("
            SELECT
                combatant_id,
                CAST(SUM(points) AS SIGNED) AS points,
                CAST(SUM(balls_thrown) AS UNSIGNED) AS balls_thrown,
                CAST(SUM(throws_hit) AS UNSIGNED) AS throws_hit,
                CAST(SUM(combatants_shoved) AS UNSIGNED) AS combatants_shoved
            FROM game_statline
            LEFT JOIN game ON game_statline.game_id = game.game_id
            WHERE game.season_id = (?)
            GROUP BY combatant_id
        ",
            self.season_id
        )
    }
}

#[derive(Debug)]
struct GetRecentGamesQuery {
    combatant_id: CombatantInstanceId,
    games_count: u32,
}

impl MySqlQuery for GetRecentGamesQuery {
    fn query(&mut self) -> impl Execute<MySql> {
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

#[tracing::instrument(skip_all)]
pub async fn get_season_stats(
    request: GetSeasonTotalsRequest,
    app_state: AppState
    // ZJ-TODO: the error type of the signature should be Result<Response, CustomError>, not NatsError
) -> Result<GetSeasonTotalsResponse, NatsError> {
    let stats = fetch_all_query!(app_state.mysql.clone(), GetSeasonTotalsQuery {
        season_id: request.season_id
    });

    let mut response = GetSeasonTotalsResponse::default();
    for row in stats {
        let combatant_id: CombatantInstanceId = row.get("combatant_id");
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

        response.combatant_statlines.insert(
            combatant_id,
            serde_json::to_vec(&statline).unwrap()
        );
    }

    Ok(response)
}

#[tracing::instrument(skip_all)]
pub async fn get_recent_stats(
    request: GetGameStatlinesRequest,
    app_state: AppState
    // ZJ-TODO: the error type of the signature should be Result<Response, CustomError>, not NatsError
) -> Result<GetGameStatlinesResponse, NatsError> {
    let combatant_id = *request.combatant_ids.first().unwrap();
    let games = fetch_all_query!(app_state.mysql.clone(), GetRecentGamesQuery {
        combatant_id,
        games_count: request.number_of_most_recent_games(),
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
                (combatant_id, serde_json::to_vec(&statline).unwrap())
            ]),
        });
    }

    Ok(response)
}