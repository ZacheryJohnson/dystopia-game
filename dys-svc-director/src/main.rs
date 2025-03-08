mod match_result;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::get, Router};
use dys_observability::logger::LoggerOptions;
use dys_observability::middleware::{make_span, map_trace_context, record_trace_id};
use dys_protocol::protocol::match_results::match_response::MatchSummary;
use dys_simulation::game::Game;
use dys_world::arena::Arena;
use dys_world::schedule::calendar::{Date, Month};
use dys_world::matches::instance::MatchInstance;
use dys_world::world::World;
use serde::{Deserialize, Serialize};

use rand::{thread_rng, SeedableRng};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use dys_datastore::datastore::Datastore;
use dys_datastore_valkey::datastore::{AsyncCommands, ValkeyConfig, ValkeyDatastore};
use crate::match_result::SummaryService;

// ZJ-TODO: this should also live elsewhere
#[derive(Clone, Serialize)]
struct CombatantTeamMember {
    team_name: String,
    combatant_name: String,
}

#[derive(Clone)]
struct WorldState {
    game_world: Arc<Mutex<World>>,
    valkey: ValkeyDatastore,
    nats: async_nats::Client,
    next_match_id: Arc<Mutex<u64>>, // ZJ-TODO: remove
}

async fn health_check(
    State(world_state): State<WorldState>
) -> Result<impl IntoResponse, StatusCode> {
    if !matches!(world_state.nats.connection_state(), async_nats::connection::State::Connected) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(StatusCode::OK)
}

#[tokio::main]
async fn main() {
    let logger_options = LoggerOptions {
        application_name: "director".to_string(),
        ..Default::default()
    };

    dys_observability::logger::initialize(logger_options);

    tracing::info!("Starting server...");

    let game_world = Arc::new(Mutex::new(dys_world::generator::Generator::new().generate_world(&mut StdRng::from_entropy())));

    let valkey_config = ValkeyConfig::new(
        std::env::var("VALKEY_USER").unwrap_or(String::from("default")),
        std::env::var("VALKEY_PASS").unwrap_or(String::from("")),
        std::env::var("VALKEY_HOST").unwrap_or(String::from("172.18.0.1")),
        std::env::var("VALKEY_PORT").unwrap_or(String::from("6379")).parse::<u16>().unwrap()
    );

    let nats_server_str = format!(
        "{}:{}",
        std::env::var("NATS_HOST").unwrap_or(String::from("172.18.0.1")),
        std::env::var("NATS_PORT").unwrap_or(String::from("4222")).parse::<u16>().unwrap(),
    );

    let nats_client = async_nats::ConnectOptions::new()
        .token(std::env::var("NATS_TOKEN").unwrap_or(String::from("replaceme")))
        .connect(nats_server_str)
        .await
        .expect("failed to connect to NATS server");

    let world_state = WorldState {
        game_world: game_world.clone(),
        valkey: *ValkeyDatastore::connect(valkey_config).await.unwrap(),
        nats: nats_client,
        next_match_id: Arc::new(Mutex::new(1)),
    };

    let world_state_thread_copy = world_state.clone();
    const SLEEP_DURATION: Duration = Duration::from_secs(5 * 60);

    tokio::spawn(async move {
        loop {
            tracing::info!("Executing simulations...");
            run_simulation(world_state_thread_copy.clone()).await;

            let mut world_state = world_state_thread_copy.clone();
            let mut valkey = world_state.valkey.connection();
            let world = world_state.game_world.lock().unwrap().to_owned();

            tracing::info!("Saving world state in valkey...");
            let _: i32 = valkey.hset(
                "env:dev:world",
                "data",
                serde_json::to_string(&world).unwrap(),
            ).await.unwrap();

            let _: i32 = valkey.expire(
                "env:dev:world",
                450,
            ).await.unwrap();

            // Sleep before simulating more matches
            tracing::info!("Sleeping for {} seconds before simulating more matches...", SLEEP_DURATION.as_secs());
            tokio::time::sleep(SLEEP_DURATION).await;
        }
    });

    let world_state_thread_copy = world_state.clone();
    tokio::spawn(async move {
        let mut summary_service = SummaryService::new(
            world_state_thread_copy.valkey.to_owned(),
            world_state_thread_copy.nats.to_owned(),
        );

        summary_service.initialize().await;

        loop {
            summary_service.process().await;
        }
    });

    let trace_middleware_layer = ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc().make_span_with(make_span))
        .map_request(map_trace_context)    
        .map_request(record_trace_id);

    let app = Router::new()
        .route("/world_state", get(get_world_state))
        .route("/health", get(health_check))
        .layer(trace_middleware_layer)
        .with_state(world_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6081").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::warn!("received ctrl+c...") },
        _ = terminate => { tracing::warn!("received terminate...") },
    }
}

fn simulate_matches(world_state: WorldState) -> Vec<MatchSummary> {
    let mut match_instances = vec![];
    let mut teams = {
        tracing::info!("Acquiring game world lock...");
        let game_world = world_state.game_world.lock().unwrap();
        game_world.teams.clone()
    };
    teams.shuffle(&mut thread_rng());

    assert_eq!(teams.len() % 2, 0);
    while !teams.is_empty() {
        let home_team = teams.pop().expect("failed to pop home team from shuffled teams list");
        let away_team = teams.pop().expect("failed to pop home team from shuffled teams list");

        match_instances.push(MatchInstance {
            match_id: world_state.next_match_id.lock().unwrap().to_owned(),
            home_team,
            away_team,
            // ZJ-TODO
            arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
            // ZJ-TODO
            date: Date(Month::Arguscorp, 1, 1)
        });

        *world_state.next_match_id.lock().unwrap() += 1;
    }

    // Simulate matches
    let mut match_results = vec![];
    for match_instance in match_instances {
        let away_team_name = match_instance.away_team.lock().unwrap().name.clone();
        let home_team_name = match_instance.home_team.lock().unwrap().name.clone();

        tracing::info!("Simulating match: {} vs {}", away_team_name, home_team_name);

        let match_id = match_instance.match_id.to_owned();
        let game = Game { match_instance };
        let game_log = game.simulate();

        match_results.push(MatchSummary {
            match_id,
            away_team_name,
            home_team_name,
            away_team_score: game_log.away_score() as u32,
            home_team_score: game_log.home_score() as u32,
            game_log_serialized: postcard::to_allocvec(&game_log).expect("failed to serialize game log"),
        });
    }

    match_results
}

#[tracing::instrument(skip_all)]
async fn run_simulation(mut world_state: WorldState) {
    let match_summary = simulate_matches(world_state.clone());

    let mut latest_ids = vec![];
    let mut valkey = world_state.valkey.connection();
    for summary in match_summary {
        latest_ids.push(summary.match_id);

        let match_summary_json = serde_json::to_string(&summary).unwrap();
        let _: i32 = valkey.hset(
            format!("env:dev:match.results:id:{}", summary.match_id),
            "data",
            match_summary_json,
        ).await.unwrap();

        let _: i32 = valkey.expire(
            format!("env:dev:match.results:id:{}", summary.match_id),
            60 * 60 // 1 hour
        ).await.unwrap();
    }

    let _: String = valkey.set(
        "env:dev:match.results:latest",
        serde_json::to_string(&latest_ids).unwrap(),
    ).await.unwrap();
}

async fn get_world_state(State(mut world_state): State<WorldState>) -> Response {
    let mut valkey = world_state.valkey.connection();
    let response_data: String = valkey.hget("env:dev:world", "data").await.unwrap();
    let mut response = response_data.into_response();
    response.headers_mut()
        // ZJ-TODO: not *
        .insert("Access-Control-Allow-Origin", HeaderValue::from_str("*").unwrap());

    response
}