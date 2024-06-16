use std::sync::{Arc, Mutex};

use criterion::{criterion_group, criterion_main, Criterion};
use dys_world::arena::{navmesh::{ArenaNavmesh, ArenaNavmeshConfig}, Arena};

fn navmesh_simulation_benchmark(c: &mut Criterion) {    
    let arena = Arc::new(Mutex::new(Arena::new_with_testing_defaults()));
    let config = ArenaNavmeshConfig::default();
    
    c.bench_function("navmesh_generation", |b| b.iter(|| {
        ArenaNavmesh::new_from(arena.clone(), config.clone());
    }));
}

criterion_group!(benches, navmesh_simulation_benchmark);
criterion_main!(benches);