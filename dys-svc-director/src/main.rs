use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::get, Router};
use dys_game::game::Game;
use dys_world::arena::Arena;
use dys_world::schedule::calendar::{Date, Month};
use dys_world::schedule::schedule_game::ScheduleGame;
use dys_world::world::World;
use serde::Serialize;

use rand::{thread_rng, RngCore};
use rand::seq::SliceRandom;

// ZJ-TODO: this should live in dys-world
#[derive(Clone, Serialize)]
struct MatchResult {
    away_team_name: String,
    home_team_name: String,
    away_team_score: u32,
    home_team_score: u32,
    game_log_serialized: Vec<u8>,
}

#[derive(Clone)]
struct WorldState {
    game_world: Arc<Mutex<World>>,
    match_results: Arc<Mutex<Vec<MatchResult>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();
    tracing::info!("Starting server...");

    let game_world = Arc::new(Mutex::new(dys_game::generator::Generator::new().generate_world()));
    
    let world_state = WorldState {
        game_world: game_world.clone(),
        match_results: Arc::new(Mutex::new(vec![]))
    };

    let world_state_thread = world_state.clone();
    tokio::spawn(async move {
        loop {
            tracing::info!("Executing simulations...");
            // Do core logic
            {
                let game_world = world_state_thread.game_world.lock().unwrap();
                
                // Generate matches
                let mut scheduled_games = vec![];
                let mut teams = game_world.teams.clone();
                teams.shuffle(&mut thread_rng());
                while !teams.is_empty() {
                    let home_team = teams.pop().expect("failed to pop home team from shuffled teams list");
                    let away_team = teams.pop().expect("failed to pop home team from shuffled teams list");

                    scheduled_games.push(ScheduleGame {
                        home_team,
                        away_team,
                        // ZJ-TODO
                        arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
                        // ZJ-TODO
                        date: Date(Month::Arguscorp, 1, 1)
                    });
                }

                // Simulate matches
                let mut match_results = vec![];
                for scheduled_game in scheduled_games {
                    let away_team_name = scheduled_game.away_team.lock().unwrap().name.clone();
                    let home_team_name = scheduled_game.home_team.lock().unwrap().name.clone();
                    let game = Game { schedule_game: scheduled_game };
                    let game_log = game.simulate();

                    let fake_away_team_score = thread_rng().next_u32() % 200;
                    let mut fake_home_team_score = thread_rng().next_u32() % 200;
                    if fake_home_team_score == fake_away_team_score {
                        fake_home_team_score += 1;
                    }

                    match_results.push(MatchResult {
                        away_team_name,
                        home_team_name,
                        away_team_score: fake_away_team_score,
                        home_team_score: fake_home_team_score,
                        game_log_serialized: postcard::to_allocvec(&game_log).expect("failed to serialize game log"),
                    });
                }

                // Swap simulation results
                // ZJ-TODO: store these in a datastore
                *world_state_thread.match_results.lock().unwrap() = match_results;
            }

            // Sleep for 60 seconds before simulating more matches
            tracing::info!("Sleeping for 60 seconds before simulating more matches...");
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    let app = Router::new()
        .route("/latest_games", get(latest_games))
        .with_state(world_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct LatestGamesResponse {
    match_results: Vec<MatchResult>,
}

async fn latest_games(State(world_state): State<WorldState>) -> Response {
    let match_results = world_state.match_results.lock().unwrap().to_owned();
    
    let mut response = 
    axum::Json(LatestGamesResponse{
        match_results,
    }).into_response();
      
    response.headers_mut()
        // ZJ-TODO: not *
        .insert("Access-Control-Allow-Origin", HeaderValue::from_str("*").unwrap());

    response
}