use std::{sync::{Arc, Mutex}, time::Duration};
use rand::prelude::StdRng;
use dys_observability::logger::LoggerOptions;
use dys_simulation::{game::Game, game_log::GameLog, generator::Generator};
use dys_world::{arena::Arena, schedule::{calendar::{Date, Month}, schedule_game::ScheduleGame}};
use tracing::Level;
use rand::{random, SeedableRng};
use rand_pcg::Pcg64;

#[tokio::main]
async fn main() {
    let logger_options = LoggerOptions {
        application_name: "testbench".to_string(),
        log_level: Level::INFO,
    };

    dys_observability::logger::initialize(logger_options);

    let seed: [u8; 32] = [13; 32];

    let generator = Generator::new();

    let mut rng = Pcg64::from_seed(seed);
    let world = generator.generate_world(&mut rng);

    let away_team = world.teams.first().expect("failed to get away team from generated world").to_owned();
    let home_team = world.teams.get(1).expect("failed to get home team from generated world").to_owned();
    let arena = Arc::new(Mutex::new(Arena::new_with_testing_defaults()));
    let date = Date(Month::Arguscorp, 1, 10000);

    let schedule_game = ScheduleGame {
        away_team,
        home_team,
        arena,
        date,
    };
    let game = Game { schedule_game };

    let game_log = game.simulate_seeded(&seed);
    let game_log_artifact = postcard::to_allocvec(&game_log).expect("failed to serialize game log");
    std::fs::write("game_log.bin", game_log_artifact).expect("failed to write game log artifact to file");

    let parsed_game_log_contents = std::fs::read("game_log.bin").expect("failed to read serialized game log artifact to vector");
    let parsed_game_log: GameLog = postcard::from_bytes(&parsed_game_log_contents).expect("failed to serialize game log artifact into a game log");
    tracing::info!("{}", parsed_game_log.perf_string());
    for tick in parsed_game_log.ticks() {
        if tick.tick_number < 100 {
            tracing::debug!("Tick {}: {}", tick.tick_number, tick.tick_performance().perf_string());
        }
        for evt in &tick.simulation_events {
            if tick.tick_number < 100 {
                tracing::debug!("\t{:?}", evt);
            }
        }
    }

    tracing::info!("H {} - {} A", game_log.home_score(), game_log.away_score());

    tokio::time::sleep(Duration::from_secs(1)).await;
}
