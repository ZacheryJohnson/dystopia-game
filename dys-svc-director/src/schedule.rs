use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use dys_nats::error::NatsError;
use dys_service_base_macros::{api, ApiRequest};
use dys_world::games::instance::GameInstanceId;
use dys_world::schedule::calendar::Date;
use dys_world::season::series::Series;
use crate::AppState;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(get_season)
)]
pub struct ScheduleApi;

#[derive(Clone, Debug, Serialize, Deserialize, ApiRequest)]
pub struct GetSeasonRequest {}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct GetSeasonResponse {
    pub season_id: u32,
    pub current_date: Date,
    pub series: Vec<Series>,
    pub game_id_to_scheduled_time_utc: HashMap<GameInstanceId, DateTime<Utc>>,
}

pub fn simulation_timings(
    first_game_time_utc: Arc<Mutex<DateTime<Utc>>>,
    series: &Vec<Series>,
) -> HashMap<GameInstanceId, DateTime<Utc>> {
    let mut simulation_timings = HashMap::new();

    let first_game_time_utc = first_game_time_utc.lock().unwrap().to_owned();
    let match_every_n_minutes = std::env::var("MINUTES_BETWEEN_MATCHES")
        .unwrap_or(String::from("15"))
        .parse::<u64>()
        .unwrap();

    // ZJ-TODO: refactor
    #[allow(unused_assignments)]
    let mut next_game_time_utc = first_game_time_utc;

    for series in series {
        for game in &series.games() {
            let game_instance = game.upgrade().unwrap();
            let days_since_first = game_instance.lock().unwrap().date.as_monotonic() - 1;
            next_game_time_utc = first_game_time_utc + Duration::from_secs(60 * match_every_n_minutes * days_since_first as u64);

            simulation_timings.insert(
                game_instance.lock().unwrap().game_id,
                next_game_time_utc
            );
        }
    }

    simulation_timings
}

#[api(
    request = GetSeasonRequest,
    app_state = AppState,
    http(
        method = "Get",
        path = "",
    ),
    nats(
        topic = "api.v1.schedule.get",
    ),
)]
pub async fn get_season(
    _: GetSeasonRequest,
    app_state: AppState,
) -> Result<GetSeasonResponse, NatsError> {
    let series = {
        let season = app_state.season.lock().unwrap();
        season.series().to_owned()
    };

    let current_date = app_state.current_date.lock().unwrap().to_owned();

    let timings = simulation_timings(
        app_state.first_game_time_utc.clone(),
        &series,
    );

    Ok(GetSeasonResponse {
        season_id: 1, // ZJ-TODO
        current_date,
        series,
        game_id_to_scheduled_time_utc: timings,
    })
}