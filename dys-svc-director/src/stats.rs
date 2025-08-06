use sqlx::{Execute, MySql, Row};
use dys_datastore_mysql::fetch_all_query;
use dys_datastore_mysql::query::MySqlQuery;
use dys_nats::error::NatsError;
use dys_protocol::nats::stats::*;
use dys_world::combatant::instance::CombatantInstanceId;
use serde::Serialize;
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