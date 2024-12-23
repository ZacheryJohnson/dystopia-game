use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{game_objects::{ball::BallState, game_object_type::GameObjectType}, game_state::GameState, game_tick::{GameTick, TickPerformance}};

use self::{ball::simulate_balls, combatant::simulate_combatants, scoring::simulate_scoring, simulation_event::SimulationEvent};

mod ball;
mod combatant;
mod scoring;
pub mod config;
pub mod simulation_event;

fn handle_collision_events(game_state: Arc<Mutex<GameState>>) -> Vec<SimulationEvent> {
    let mut new_simulation_events = vec![];

    let (collision_events, active_colliders, balls) = {
        let mut game_state = game_state.lock().unwrap();

        let collision_events = game_state.physics_sim.collision_events().to_owned();
        let active_colliders = game_state.active_colliders.to_owned();
        let balls = game_state.balls.to_owned();

        (collision_events, active_colliders, balls)
    };

    while let Ok(evt) = collision_events.try_recv() {
        let maybe_collider_1 = active_colliders.get(&evt.collider1());
        let maybe_collider_2 = active_colliders.get(&evt.collider2());
        if maybe_collider_1.is_none() || maybe_collider_2.is_none() {
            continue;
        }

        let collider_1 = maybe_collider_1.unwrap();
        let collider_2 = maybe_collider_2.unwrap();

        match (collider_1, collider_2) {
            (GameObjectType::Invalid, _) | (_, GameObjectType::Invalid) => continue,
            (GameObjectType::Ball(ball_id), GameObjectType::Barrier) | (GameObjectType::Barrier, GameObjectType::Ball(ball_id)) => {
                // We don't care if the ball and barrier have stopped colliding
                if evt.stopped() {
                    continue;
                }

                let ball_obj = balls.get(ball_id).expect("Received invalid ball ID");

                match ball_obj.state {
                    BallState::ThrownAtTarget { direction: _direction, thrower_id, target_id } => {
                        // ZJ-TODO: do this in simulation: Ball aiming for a target hit the arena - mark it as rolling now
                        new_simulation_events.push(SimulationEvent::BallCollisionArena { thrower_id, original_target_id: target_id, ball_id: *ball_id });
                    },
                    _ => ()
                }
            },
            (GameObjectType::Plate(plate_id), GameObjectType::Combatant(combatant_id)) | (GameObjectType::Combatant(combatant_id), GameObjectType::Plate(plate_id)) => {
                if evt.started() {
                    new_simulation_events.push(SimulationEvent::CombatantOnPlate { combatant_id: *combatant_id, plate_id: *plate_id })
                }

                if evt.stopped() {
                    new_simulation_events.push(SimulationEvent::CombatantOffPlate { combatant_id: *combatant_id, plate_id: *plate_id })
                }
            },
            (GameObjectType::Plate(_), _) | (_, GameObjectType::Plate(_)) => continue,
            (GameObjectType::BallSpawn, _) | (_, GameObjectType::BallSpawn) => continue,
            (GameObjectType::Barrier, _) | (_, GameObjectType::Barrier) => continue,
            (GameObjectType::Ball(_), GameObjectType::Ball(_)) => continue,
            (GameObjectType::Combatant(_), GameObjectType::Combatant(_)) => continue,
            (GameObjectType::Ball(ball_id), GameObjectType::Combatant(combatant_id)) | (GameObjectType::Combatant(combatant_id), GameObjectType::Ball(ball_id)) => {
                // We don't care if the ball and combatant have stopped colliding
                if evt.stopped() {
                    continue;
                }

                let ball_obj = balls.get(ball_id).expect("Received invalid ball ID");
            
                match ball_obj.state {
                    BallState::ThrownAtTarget { direction: _, thrower_id, target_id: _ } => {
                        // ZJ-TODO: check team of hit combatant, and only explode if enemy
                        new_simulation_events.push(SimulationEvent::BallCollisionEnemy { thrower_id, enemy_id: *combatant_id, ball_id: *ball_id });
                    },
                    _ => ()
                }
            }
        }
    }

    new_simulation_events
}

pub fn simulate_tick(game_state: Arc<Mutex<GameState>>) -> GameTick {
    // ZJ-TODO: make SimulationStage struct

    let pre_tick_timestamp = Instant::now();

    let (current_tick, simulation_config) = {
        let mut game_state = game_state.lock().unwrap();

        game_state.current_tick += 1;
        game_state.physics_sim.tick();

        (game_state.current_tick, game_state.simulation_config.clone())
    };


    let is_halftime = current_tick == simulation_config.ticks_per_half();
    let is_end_of_game = current_tick == simulation_config.ticks_per_game();
    let is_scoring_tick = current_tick % simulation_config.ticks_per_second() == 0;
    let mut simulation_events = vec![];

    // ZJ-TODO: move event handling elsewhere
    let collision_simulation_events = handle_collision_events(game_state.clone());
    simulation_events.extend(collision_simulation_events);

    let pre_balls_timestamp = Instant::now();
    let ball_simulation_events = simulate_balls(game_state.clone());
    simulation_events.extend(ball_simulation_events);
    let post_balls_timestamp = Instant::now();

    let pre_combatant_timestamp = Instant::now();
    let combatant_simulation_events = simulate_combatants(game_state.clone());
    simulation_events.extend(combatant_simulation_events);
    let post_combatant_timestamp = Instant::now();

    let pre_scoring_timestamp = Instant::now();
    if is_scoring_tick {
        let scoring_simulation_events = simulate_scoring(game_state.clone());
        simulation_events.extend(scoring_simulation_events);
    }
    let post_scoring_timestamp = Instant::now();

    let post_tick_timestamp = Instant::now();

    GameTick {
        tick_number: current_tick,
        tick_performance: TickPerformance::new(
            Duration::from_secs(0), // ZJ-TODO
            post_balls_timestamp - pre_balls_timestamp,
            post_combatant_timestamp - pre_combatant_timestamp,
            post_scoring_timestamp - pre_scoring_timestamp,
            post_tick_timestamp - pre_tick_timestamp
        ),
        simulation_events,
        is_halftime,
        is_end_of_game
    }
}