use std::time::Instant;

use crate::{game_objects::{ball::BallState, game_object_type::GameObjectType}, game_state::GameState, game_tick::GameTick};

use self::{ball::simulate_balls, combatant::simulate_combatants};

mod ball;
mod combatant;
mod simulation_event;

// TODO: config driven?
const TICKS_PER_SECOND: u32 = 5;
const SECONDS_PER_HALF: u32 = 60 * 5;
const TICKS_PER_HALF: u32   = SECONDS_PER_HALF * TICKS_PER_SECOND;
const TICKS_PER_GAME: u32   = TICKS_PER_HALF * 2;

fn handle_collision_events(game_state: &mut GameState) {
    let collision_events =  game_state.physics_sim.collision_events();
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
            (GameObjectType::Wall, _) | (_, GameObjectType::Wall) => continue,
            (GameObjectType::Ball(_), GameObjectType::Ball(_)) => continue,
            (GameObjectType::Combatant(_), GameObjectType::Combatant(_)) => continue,
            (GameObjectType::Ball(ball_id), GameObjectType::Combatant(combatant_id)) | (GameObjectType::Combatant(combatant_id), GameObjectType::Ball(ball_id)) => {
                let ball_obj = game_state.balls.get_mut(ball_id).expect("Received invalid ball ID");
                let combatant_obj = game_state.combatants.get(combatant_id).expect("Received invalid combatant ID");
            
                match ball_obj.state {
                    BallState::ThrownAtTarget { direction, velocity, thrower_id, target_id } => {
                        
                    },
                    _ => ()
                }

                let (old_state, old_state_tick) = ball_obj.change_state(game_state.current_tick, BallState::Explode);
            }
        }
    }
}

pub fn simulate_tick(game_state: &mut GameState) -> GameTick {
    // ZJ-TODO: a macro (like `tick_duration = duration! { ... rest of function ... }`) would be neat

    let pre_tick_timestamp = Instant::now();
    
    game_state.current_tick += 1;
    let is_halftime = game_state.current_tick == TICKS_PER_HALF;
    let is_end_of_game = game_state.current_tick == TICKS_PER_GAME;

    let pre_physics_timestamp = Instant::now();
    game_state.physics_sim.tick();

    // ZJ-TODO: move event handling elsewhere
    handle_collision_events(game_state);

    let post_physics_timestamp = Instant::now();

    let pre_balls_timestamp = Instant::now();
    simulate_balls(game_state);
    let post_balls_timestamp = Instant::now();

    let pre_combatant_timestamp = Instant::now();
    simulate_combatants(game_state);
    let post_combatant_timestamp = Instant::now();

    let post_tick_timestamp = Instant::now();

    GameTick {
        tick_number: game_state.current_tick,
        physics_duration: post_physics_timestamp - pre_physics_timestamp,
        balls_duration: post_balls_timestamp - pre_balls_timestamp,
        tick_duration: post_tick_timestamp - pre_tick_timestamp,
        is_halftime,
        is_end_of_game,
    }
}