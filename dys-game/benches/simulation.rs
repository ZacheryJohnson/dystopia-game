use std::sync::{Arc, Mutex};

use criterion::{criterion_group, criterion_main, Criterion};
use dys_game::game::Game;
use dys_world::{arena::Arena, schedule::{calendar::{Date, Month}, schedule_game::ScheduleGame}, team::team::Team};

fn game_simulation_benchmark(c: &mut Criterion) {
    let game = Game {
        schedule_game: ScheduleGame {
            away_team: Arc::new(Mutex::new(Team {
                id: 1,
                name: String::from("Away Team"),
                combatants: vec![],
            })),
            home_team: Arc::new(Mutex::new(Team {
                id: 2,
                name: String::from("Home Team"),
                combatants: vec![],
            })),
            arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
            date: Date(Month::Arguscorp, 1, 10000),
        },
    };
    let seed = &[0; 32];
    
    c.bench_function("full game simulation", |b| b.iter(|| game.simulate_seeded(seed)));
}

criterion_group!(benches, game_simulation_benchmark);
criterion_main!(benches);