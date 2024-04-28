use std::sync::{Arc, Mutex};
use dys_game::{game::Game, generator::Generator};
use dys_world::{arena::Arena, schedule::{calendar::{Date, Month}, schedule_game::ScheduleGame}};

fn main() {
    let generator = Generator::new();
    let world = generator.generate_world();

    let away_team = world.teams.get(0).expect("failed to get away team from generated world").to_owned();
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
    for tick in game_log.ticks {
        if tick.tick_number > 25 {
            continue;
        }

        println!("Tick {}: {}", tick.tick_number, tick.perf_string());

        for simulation_evt in tick.simulation_events {
            println!("\t{simulation_evt:?}");
        }
    }
}
