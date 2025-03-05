use std::sync::{Arc, Mutex};

use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::StdRng;
use rand::SeedableRng;
use dys_simulation::{game::Game, generator::Generator};
use dys_world::{arena::Arena, schedule::{calendar::{Date, Month}, instance::ScheduleGame}};

fn game_simulation_benchmark(c: &mut Criterion) {
    let world = Generator::new().generate_world(&mut StdRng::from_entropy());
    let game = Game {
        schedule_game: ScheduleGame {
            away_team: world.teams[0].clone(),
            home_team: world.teams[1].clone(),
            arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
            date: Date(Month::Arguscorp, 1, 10000),
        },
    };
    let seed = &[0; 32];
    
    c.bench_function("full_game_simulation", |b| b.iter(|| game.simulate_seeded(seed)));
}

criterion_group!(benches, game_simulation_benchmark);
criterion_main!(benches);