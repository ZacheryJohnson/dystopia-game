use std::time::Instant;

use crate::{game_state::GameState, game_tick::GameTick};

use self::{ball::simulate_balls, combatant::simulate_combatants};

mod ball;
mod combatant;

// TODO: config driven?
const TICKS_PER_SECOND: u32 = 5;
const SECONDS_PER_HALF: u32 = 60 * 5;
const TICKS_PER_HALF: u32   = SECONDS_PER_HALF * TICKS_PER_SECOND;
const TICKS_PER_GAME: u32   = TICKS_PER_HALF * 2;

pub fn simulate_tick(game_state: &mut GameState) -> GameTick {
    let pre_tick_timestamp = Instant::now();
    
    game_state.current_tick += 1;
    let is_halftime = game_state.current_tick == TICKS_PER_HALF;
    let is_end_of_game = game_state.current_tick == TICKS_PER_GAME;

    let pre_physics_timestamp = Instant::now();
    game_state.physics_sim.tick();
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