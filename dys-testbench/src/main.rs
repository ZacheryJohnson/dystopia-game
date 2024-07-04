use std::sync::{Arc, Mutex};
use dys_game::{game::Game, game_log::GameLog, generator::Generator};
use dys_world::{arena::Arena, schedule::{calendar::{Date, Month}, schedule_game::ScheduleGame}};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

fn register_tracing_subscriber() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "dystopia".into(), 
        std::io::stdout
    );
    
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    
    set_global_default(subscriber).expect("Failed to set subscriber");
}

fn main() {
    register_tracing_subscriber();

    let generator = Generator::new();
    let world = generator.generate_world();

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
    let seed: [u8; 32] = [13; 32];

    let game_log = game.simulate_seeded(&seed);
    let game_log_artifact = postcard::to_allocvec(&game_log).expect("failed to serialize game log");
    std::fs::write("game_log.bin", game_log_artifact).expect("failed to write game log artifact to file");

    let parsed_game_log_contents = std::fs::read("game_log.bin").expect("failed to read serialized game log artifact to vector");
    let parsed_game_log: GameLog = postcard::from_bytes(&parsed_game_log_contents).expect("failed to serialize game log artifact into a game log");
    tracing::info!("{}", parsed_game_log.perf_string());
    for tick in parsed_game_log.ticks() {
        tracing::info!("Tick {}: {}", tick.tick_number, tick.tick_performance().perf_string());
        for evt in &tick.simulation_events {
            tracing::info!("\t{:?}", evt);
        }
    }
}
