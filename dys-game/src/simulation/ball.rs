use crate::{game_objects::ball::BallObject, game_state::GameState, game_tick::GameTickNumber};

use super::TICKS_PER_SECOND;

const BALL_CHARGE_DECAY_PER_TICK: f32 = 2.0;
const BALL_MAX_CHARGE: f32 = 100.0;
const BALL_MAX_HOLD_TIME_SECONDS: u32 = 7;
const BALL_MAX_HOLD_TIME_TICKS: u32 = TICKS_PER_SECOND * BALL_MAX_HOLD_TIME_SECONDS;

pub(crate) fn simulate_balls(game_state: &mut GameState) {
    for (_, ball) in &mut game_state.balls {
        move_ball(ball, game_state.current_tick);
        explode(ball);
        decrease_charge(ball);
    }
}

fn move_ball(ball: &mut BallObject, current_tick: GameTickNumber) {
    match &ball.state {
        crate::game_objects::ball::BallState::Idle => {},
        crate::game_objects::ball::BallState::Held { holder_id } => {
            // TODO: handle ball exploding
            let should_explode_on_holder = (current_tick - ball.state_tick_stamp)  >= BALL_MAX_HOLD_TIME_TICKS;
        },
        crate::game_objects::ball::BallState::RollingInDirection { direction, velocity } => {
            // TODO: move ball in direction
            // TODO: detect collisions
        },
        crate::game_objects::ball::BallState::ThrownAtTarget { direction, velocity, thrower_id, target_id } => {
            // TODO: move ball in direction
            // TODO: detect collisions
        },
        crate::game_objects::ball::BallState::Explode => {
            // no-op: explosion detected in another function
        },
    }
}

fn explode(ball: &mut BallObject) {
    let should_explode = match &ball.state {
        crate::game_objects::ball::BallState::Explode => true,
        _ => false,
    };

    if !should_explode {
        return;
    }

    // TODO: handle explosion
}

fn decrease_charge(ball: &mut BallObject) {
    ball.charge = (ball.charge - BALL_CHARGE_DECAY_PER_TICK).clamp(0.0, BALL_MAX_CHARGE);
}