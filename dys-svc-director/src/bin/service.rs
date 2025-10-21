use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::{DateTime, Timelike, Utc};
use rand::prelude::StdRng;
use rand::SeedableRng;
use sqlx::mysql::MySqlConnectOptions;
use tokio::time::Instant;
use director::{run_simulation, AppState};
use director::schedule::simulation_timings;
use director::world_old::generate_world;
use dys_datastore::datastore::Datastore;
use dys_datastore_mysql::datastore::MySqlDatastore;
use dys_datastore_valkey::datastore::{AsyncCommands, ValkeyConfig, ValkeyDatastore};
use dys_nats::rpc::router::NatsRouter;
use dys_observability::logger::LoggerOptions;
use dys_world::schedule::calendar::Date;
use dys_world::schedule::calendar::Month::Arguscorp;

#[tokio::main]
async fn main() {
    let logger_options = LoggerOptions {
        application_name: "director".to_string(),
        ..Default::default()
    };

    dys_observability::logger::initialize(logger_options);

    tracing::info!("Starting server...");

    let (game_world, season) = generate_world().await;

    let valkey_config = ValkeyConfig::new(
        std::env::var("VALKEY_USER").unwrap_or(String::from("default")),
        std::env::var("VALKEY_PASS").unwrap_or(String::from("")),
        std::env::var("VALKEY_HOST").unwrap_or(String::from("172.18.0.1")),
        std::env::var("VALKEY_PORT").unwrap_or(String::from("6379")).parse::<u16>().unwrap()
    );

    let mysql_config = MySqlConnectOptions::new()
        .host(&std::env::var("MYSQL_HOST").unwrap_or(String::from("127.0.0.1")))
        .username(&std::env::var("MYSQL_USER").unwrap_or(String::from("default")))
        .password(&std::env::var("MYSQL_PASS").unwrap_or(String::from("")))
        .port(std::env::var("MYSQL_PORT").unwrap_or(String::from("3306")).parse::<u16>().unwrap())
        .database(&std::env::var("MYSQL_DATABASE").unwrap_or(String::from("")));

    let mysql = Arc::new(Mutex::new(
        MySqlDatastore::connect(mysql_config).await.unwrap()
    ));

    director::world_old::save_world(mysql.clone(), game_world.clone(), &season).await;

    // Get first match time
    let first_game_time_utc = {
        let match_every_n_minutes = std::env::var("MINUTES_BETWEEN_MATCHES")
            .unwrap_or(String::from("15"))
            .parse::<u64>()
            .unwrap();

        let now_utc = Utc::now();
        let second_adjustment = 60 - now_utc.second() as u64 % 60;
        let second_adjusted_utc = now_utc + Duration::from_secs(second_adjustment);

        let minute_adjustment = match_every_n_minutes - second_adjusted_utc.minute() as u64 % match_every_n_minutes;
        second_adjusted_utc + Duration::from_secs(60 * minute_adjustment)
    };

    let app_state = AppState {
        game_world: game_world.clone(),
        season: Arc::new(Mutex::new(season)),
        current_date: Arc::new(Mutex::new(Date::new(
            Arguscorp, 1, 10000
        ))),
        first_game_time_utc: Arc::new(Mutex::new(first_game_time_utc)),
        valkey: Arc::new(Mutex::new(
            ValkeyDatastore::connect(valkey_config).await.unwrap()
        )),
        mysql,
    };

    let app_state_thread_copy = app_state.clone();
    tokio::task::spawn(async move {
        loop {
            let app_state = app_state_thread_copy.clone();
            let mut valkey = app_state.valkey.lock().unwrap().connection();
            let world = app_state.game_world.lock().unwrap().to_owned();

            tracing::info!("Saving world state in valkey...");
            let _: i32 = valkey.hset(
                "env:dev:world",
                "data",
                serde_json::to_string(&world).unwrap(),
            ).await.unwrap();

            let _: i32 = valkey.expire(
                "env:dev:world",
                60 * 60, // 1 hour
            ).await.unwrap();

            // Generate new proposals for the upcoming games
            let generator = dys_world::generator::Generator::new();
            let proposals = generator.generate_proposals(&mut StdRng::from_os_rng(), &world);

            let mut zj_todo_id = 1;
            for proposal in proposals {
                let _: i32 = valkey.hset(
                    "env:dev:proposals:latest",
                    zj_todo_id.to_string(),
                    serde_json::to_string(&proposal).unwrap(),
                ).await.unwrap();

                zj_todo_id += 1;
            }

            // Sleep before simulating more games
            let next_time = {
                let season = app_state.season.lock().unwrap();
                simulation_timings(app_state.first_game_time_utc, season.series())
                    .values()
                    .filter(|time| time.timestamp() >= Utc::now().timestamp())
                    .min()
                    .map_or(DateTime::default(), |time| time.to_owned())
            };

            let offset = next_time.timestamp() - Utc::now().timestamp();
            let instant = Instant::now() + Duration::from_secs(offset as u64);
            tracing::info!("Sleeping until {} before simulating more games...", next_time.to_rfc3339());
            tokio::time::sleep_until(instant).await;

            tracing::info!("Executing simulations...");
            run_simulation(app_state_thread_copy.clone()).await;
        }
    });

    let recent = director::stats::recent::nats::GetRecentStatsNatsService::from(app_state.clone());
    let recent_topic = recent.topic.clone();

    let season = director::stats::season::nats::GetSeasonStatsNatsService::from(app_state.clone());
    let season_topic = season.topic.clone();

    let world_state = director::world::state::nats::GetWorldStateNatsService::from(app_state.clone());
    let world_state_topic = world_state.topic.clone();

    let game_result = director::game::summary::nats::GetSummariesNatsService::from(app_state.clone());
    let game_result_topic = game_result.topic.clone();

    let log = director::game::log::nats::GetGameLogNatsService::from(app_state.clone());
    let log_topic = log.topic.clone();

    let schedule = director::schedule::nats::GetSeasonNatsService::from(app_state.clone());
    let schedule_topic = schedule.topic.clone();

    let vote = director::vote::proposal::nats::GetVotingProposalsNatsService::from(app_state.clone());
    let vote_topic = vote.topic.clone();

    let submit = director::vote::submit::nats::SubmitVoteNatsService::from(app_state.clone());
    let submit_topic = submit.topic.clone();

    let nats = NatsRouter::new()
        .await
        .service_2(recent, recent_topic)
        .service_2(season, season_topic)
        .service_2(world_state, world_state_topic)
        .service_2(game_result, game_result_topic)
        .service_2(log, log_topic)
        .service_2(schedule, schedule_topic)
        .service_2(vote, vote_topic)
        .service_2(submit, submit_topic);

    nats.run().await;
}