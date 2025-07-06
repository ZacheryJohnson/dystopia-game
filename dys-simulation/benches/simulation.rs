use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::StdRng;
use rand::SeedableRng;
use dys_simulation::game::Game;
use dys_world::{schedule::{calendar::{Date, Month}}, matches::instance::MatchInstance, generator::Generator};

fn game_simulation_benchmark(c: &mut Criterion) {
    // These tests take a while, so use a smaller sample size
    // Default is 100
    let mut group = c.benchmark_group("simulation");
    group.sample_size(20);

    let world = Generator::new().generate_world(&mut StdRng::from_os_rng());
    let game = Game {
        match_instance: MatchInstance {
            match_id: 0,
            away_team: world.teams[0].clone(),
            home_team: world.teams[1].clone(),
            // arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
            arena_id: 0,
            date: Date(Month::Arguscorp, 1, 10000),
        },
    };
    let seed = &[0; 32];
    
    group.bench_function("full_game_simulation", |b| b.iter(|| game.simulate_seeded(seed)));
    group.finish();
}

criterion_group!(benches, game_simulation_benchmark);
criterion_main!(benches);