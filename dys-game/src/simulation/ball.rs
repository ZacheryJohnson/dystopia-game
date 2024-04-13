use crate::{game_objects::{ball::{BallObject, BallState}, game_object::GameObject}, game_state::GameState};

use super::{simulation_event::SimulationEvent, TICKS_PER_SECOND};

const BALL_CHARGE_DECAY_PER_TICK: f32 = 2.0;
const BALL_MAX_CHARGE: f32 = 100.0;
const BALL_MAX_HOLD_TIME_SECONDS: u32 = 7;
const BALL_MAX_HOLD_TIME_TICKS: u32 = TICKS_PER_SECOND * BALL_MAX_HOLD_TIME_SECONDS;

pub(crate) fn simulate_balls(game_state: &mut GameState) -> Vec<SimulationEvent> {
    let mut events = vec![];

    for (ball_id, ball_object) in &mut game_state.balls {
        explode(ball_object);
        decrease_charge(ball_object);
        
        let ball_rb_handle = ball_object.rigid_body_handle().expect("ball should have a valid rigidbody handle");

        let (rigid_body_set, _) = game_state.physics_sim.sets();

        let ball_rb = rigid_body_set.get(ball_rb_handle).expect("ball rigid bodies should be registered with main set");
        events.push(SimulationEvent::BallPositionUpdate { ball_id: *ball_id, position: *ball_rb.translation() });
    }

    events
}

fn explode(ball: &mut BallObject) {
    let should_explode = match &ball.state {
        BallState::Explode => true,
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