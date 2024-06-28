use std::time::Instant;

use crate::{game_objects::{ball::BallState, game_object::GameObject, game_object_type::GameObjectType}, game_state::GameState, game_tick::{GameTick, TickPerformance}};

use self::{ball::simulate_balls, combatant::simulate_combatants, scoring::simulate_scoring, simulation_event::SimulationEvent};

mod ball;
mod combatant;
mod scoring;
pub mod config;
pub mod simulation_event;

fn handle_collision_events(game_state: &mut GameState) -> Vec<SimulationEvent> {
    let mut new_simulation_events = vec![];

    let collision_events =  game_state.physics_sim.collision_events().to_owned();
    while let Ok(evt) = collision_events.try_recv() {
        let maybe_collider_1 = game_state.active_colliders.get(&evt.collider1());
        let maybe_collider_2 = game_state.active_colliders.get(&evt.collider2());
        if maybe_collider_1.is_none() || maybe_collider_2.is_none() {
            continue;
        }

        let collider_1 = maybe_collider_1.unwrap();
        let collider_2 = maybe_collider_2.unwrap();

        match (collider_1, collider_2) {
            (GameObjectType::Invalid, _) | (_, GameObjectType::Invalid) => continue,
            (GameObjectType::Ball(ball_id), GameObjectType::Barrier) | (GameObjectType::Barrier, GameObjectType::Ball(ball_id)) => {
                let ball_obj = game_state.balls.get_mut(ball_id).expect("Received invalid ball ID");

                match ball_obj.state {
                    BallState::ThrownAtTarget { direction, thrower_id, target_id } => {
                        // Ball aiming for a target hit the arena - mark it as rolling now
                        let (_old_state, _old_state_tick) = ball_obj.change_state(game_state.current_tick, BallState::RollingInDirection { direction });
                        new_simulation_events.push(SimulationEvent::BallCollisionArena { thrower_id, original_target_id: target_id, ball_id: *ball_id });
                    },
                    _ => ()
                }
            },
            (GameObjectType::Plate(_), _) | (_, GameObjectType::Plate(_)) => continue, // ZJ-TODO: should we flag players as on/off plates rather than checking at scoring time?
            (GameObjectType::BallSpawn, _) | (_, GameObjectType::BallSpawn) => continue,
            (GameObjectType::Barrier, _) | (_, GameObjectType::Barrier) => continue,
            (GameObjectType::Ball(_), GameObjectType::Ball(_)) => continue,
            (GameObjectType::Combatant(_), GameObjectType::Combatant(_)) => continue,
            (GameObjectType::Ball(ball_id), GameObjectType::Combatant(combatant_id)) | (GameObjectType::Combatant(combatant_id), GameObjectType::Ball(ball_id)) => {
                let ball_obj = game_state.balls.get_mut(ball_id).expect("Received invalid ball ID");
                let _combatant_obj = game_state.combatants.get(combatant_id).expect("Received invalid combatant ID");
            
                match ball_obj.state {
                    BallState::ThrownAtTarget { direction: _, thrower_id, target_id: _ } => {
                        // ZJ-TODO: check team of hit combatant, and only explode if enemy
                        let (_old_state, _old_state_tick) = ball_obj.change_state(game_state.current_tick, BallState::Explode);
                        new_simulation_events.push(SimulationEvent::BallCollisionEnemy { thrower_id, enemy_id: *combatant_id, ball_id: *ball_id });
                    },
                    _ => ()
                }
            }
        }
    }

    new_simulation_events
}

pub fn simulate_tick(game_state: &mut GameState) -> GameTick {
    // ZJ-TODO: a macro (like `tick_duration = duration! { ... rest of function ... }`) would be neat

    let pre_tick_timestamp = Instant::now();
    
    game_state.current_tick += 1;

    let simulation_config = &game_state.simulation_config;
    let is_halftime = game_state.current_tick == simulation_config.ticks_per_half();
    let is_end_of_game = game_state.current_tick == simulation_config.ticks_per_game();
    let is_scoring_tick = game_state.current_tick % simulation_config.ticks_per_second() == 0;
    let mut simulation_events = vec![];

    let pre_physics_timestamp = Instant::now();
    game_state.physics_sim.tick();

    // ZJ-TODO: move event handling elsewhere
    let collision_simulation_events = handle_collision_events(game_state);
    simulation_events.extend(collision_simulation_events);

    let post_physics_timestamp = Instant::now();

    let pre_balls_timestamp = Instant::now();
    let ball_simulation_events = simulate_balls(game_state);
    simulation_events.extend(ball_simulation_events);
    let post_balls_timestamp = Instant::now();

    let pre_combatant_timestamp = Instant::now();
    let combatant_simulation_events = simulate_combatants(game_state);
    simulation_events.extend(combatant_simulation_events);
    let post_combatant_timestamp = Instant::now();

    let pre_scoring_timestamp = Instant::now();
    if is_scoring_tick {
        let scoring_simulation_events = simulate_scoring(game_state);
        simulation_events.extend(scoring_simulation_events);
    }
    let post_scoring_timestamp = Instant::now();

    let post_tick_timestamp = Instant::now();

    GameTick {
        tick_number: game_state.current_tick,
        tick_performance: TickPerformance::new(
            post_physics_timestamp - pre_physics_timestamp,
            post_balls_timestamp - pre_balls_timestamp,
            post_combatant_timestamp - pre_combatant_timestamp,
            post_scoring_timestamp - pre_scoring_timestamp,
            post_tick_timestamp - pre_tick_timestamp
        ),
        simulation_events,
        is_halftime,
        is_end_of_game,
        is_scoring_tick,
    }
}