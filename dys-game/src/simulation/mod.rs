use crate::{game_state::GameState, game_tick::GameTick};

use self::ball::simulate_balls;

mod ball;

// TODO: config driven?
const TICKS_PER_SECOND: u32 = 5;
const SECONDS_PER_HALF: u32 = 60 * 5;
const TICKS_PER_HALF: u32   = SECONDS_PER_HALF * TICKS_PER_SECOND;
const TICKS_PER_GAME: u32   = TICKS_PER_HALF * 2;

pub fn simulate_tick(game_state: &mut GameState) -> GameTick {
    game_state.current_tick += 1;

    let is_halftime = game_state.current_tick == TICKS_PER_HALF;
    let is_end_of_game = game_state.current_tick == TICKS_PER_GAME;

    simulate_balls(game_state);

    GameTick {
        tick_number: game_state.current_tick,
        is_halftime,
        is_end_of_game,
    }
}