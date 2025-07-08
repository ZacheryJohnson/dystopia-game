use std::{sync::{Arc, Mutex}, time::Duration};
use dys_observability::logger::LoggerOptions;
use dys_simulation::{game::Game, game_log::GameLog};
use dys_world::{
    arena::Arena,
    generator::Generator,
    matches::instance::MatchInstance,
    schedule::{calendar::{Date, Month}},
};
use tracing::Level;
use rand::SeedableRng;
use rand_pcg::Pcg64;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let log_level = if args.contains(&"--debug".to_string()) {
        Level::DEBUG
    } else if args.contains(&"--trace".to_string()) {
        Level::TRACE
    } else {
        Level::INFO
    };

    let logger_options = LoggerOptions {
        application_name: "testbench".to_string(),
        log_level,
        with_ansi: !args.contains(&"--no-ansi".to_string()),
    };

    dys_observability::logger::initialize(logger_options);

    let seed: [u8; 32] = [13; 32];

    let generator = Generator::new();

    let mut rng = Pcg64::from_seed(seed);
    let world = generator.generate_world(&mut rng);

    let away_team = world.teams.first().expect("failed to get away team from generated world").to_owned();
    let home_team = world.teams.get(1).expect("failed to get home team from generated world").to_owned();
    let _arena = Arc::new(Mutex::new(Arena::new_with_testing_defaults()));
    let date = Date(Month::Arguscorp, 1, 10000);

    let match_instance = MatchInstance {
        match_id: 0,
        away_team,
        home_team,
        // arena,
        arena_id: 0,
        date,
    };
    let game = Game { match_instance };

    let game_log = game.simulate_seeded(&seed);
    let game_log_artifact = postcard::to_allocvec(&game_log).expect("failed to serialize game log");
    std::fs::write("game_log.bin", game_log_artifact).expect("failed to write game log artifact to file");

    let world_artifact = serde_json::to_string(&world).expect("failed to serialize world artifact");
    std::fs::write("world_state.bin", world_artifact).expect("failed to write world artifact to file");

    let parsed_game_log_contents = std::fs::read("game_log.bin").expect("failed to read serialized game log artifact to vector");
    let parsed_game_log: GameLog = postcard::from_bytes(&parsed_game_log_contents).expect("failed to serialize game log artifact into a game log");
    tracing::info!("{}", parsed_game_log.perf_string());
    for tick in parsed_game_log.ticks() {
        tracing::debug!("Tick {}: {}", tick.tick_number, tick.tick_performance().perf_string());

        for evt in &tick.simulation_events {
            tracing::debug!("\t{:?}", evt);
        }
    }

    tracing::info!("H {} - {} A", game_log.home_score(), game_log.away_score());

    tokio::time::sleep(Duration::from_secs(1)).await;
}
