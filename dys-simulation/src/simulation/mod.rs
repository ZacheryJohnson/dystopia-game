use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{game_state::GameState, game_tick::{GameTick, TickPerformance}};
use crate::simulation::collision::handle_collision_events;
use crate::simulation::simulation_event::SimulationEvent;
use crate::simulation::simulation_stage::SimulationStage;
use self::{ball::simulate_balls, combatant::simulate_combatants, scoring::simulate_scoring};

mod ball;
mod collision;
mod combatant;
mod scoring;
mod simulation_stage;

pub mod config;
pub mod simulation_event;

pub fn simulate_tick(game_state: Arc<Mutex<GameState>>) -> GameTick {
    let pre_tick_timestamp = Instant::now();

    let (current_tick, simulation_config, phys_duration, highest_score) = {
        let mut game_state = game_state.lock().unwrap();

        game_state.current_tick += 1;

        let pre_tick_timestamp = Instant::now();
        game_state.physics_sim.tick();
        let post_tick_timestamp = Instant::now();

        let highest_score = game_state.home_points.max(game_state.away_points);

        (game_state.current_tick, game_state.simulation_config.clone(), post_tick_timestamp - pre_tick_timestamp, highest_score)
    };

    let is_halftime = current_tick == simulation_config.ticks_per_half();
    let is_end_of_game = current_tick == simulation_config.ticks_per_game() || highest_score >= simulation_config.game_conclusion_score();
    let is_scoring_tick = current_tick % simulation_config.ticks_per_second() == 0;

    let mut pending_simulation_events = vec![];

    let collision_stage = handle_collision_events(game_state.clone());
    let balls_stage = simulate_balls(game_state.clone());
    let combatants_stage = simulate_combatants(game_state.clone());
    let scoring_stage = if is_scoring_tick {
        simulate_scoring(game_state.clone())
    } else {
        SimulationStage { execution_duration: Duration::new(0, 0), pending_events: vec![] }
    };

    pending_simulation_events.extend(collision_stage.pending_events);
    pending_simulation_events.extend(balls_stage.pending_events);
    pending_simulation_events.extend(combatants_stage.pending_events);
    pending_simulation_events.extend(scoring_stage.pending_events);

    let mut simulation_events = vec![];

    for pending_event in pending_simulation_events {
        if !SimulationEvent::simulate_event(game_state.clone(), &pending_event) {
            tracing::warn!("failed to simulate pending event: {:?}", pending_event);
            continue;
        }

        simulation_events.push(pending_event);
    }

    let post_tick_timestamp = Instant::now();

    GameTick {
        tick_number: current_tick,
        tick_performance: TickPerformance::new(
            phys_duration,
            balls_stage.execution_duration,
            combatants_stage.execution_duration,
            scoring_stage.execution_duration,
            post_tick_timestamp - pre_tick_timestamp
        ),
        simulation_events,
        is_halftime,
        is_end_of_game
    }
}