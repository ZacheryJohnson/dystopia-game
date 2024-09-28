use rapier3d::{na::{Point3, Vector3}, prelude::*};

use crate::{ai::{agent::Agent, belief::Belief, strategy::Strategy}, game_state::GameState, simulation::simulation_event::SimulationEvent};

pub struct ThrowBallAtTargetLocationStrategy {
    target_location: Point3<f32>,
}

impl Strategy for ThrowBallAtTargetLocationStrategy {
    fn name(&self) -> String {
        String::from("Throw Ball at Target Location")
    }

    fn can_perform(&self, owned_beliefs: &[Belief]) -> bool {
        owned_beliefs.contains(&Belief::SelfHasBall)
    }

    fn is_complete(&self) -> bool {
        todo!()
    }

    fn tick(&mut self, agent: &mut dyn Agent, game_state: &mut GameState) -> Vec<SimulationEvent> {
        // ZJ-TODO: wait for some delay to simulate a "windup" for a throw - should we allow movement in a direction (eg crow hop)?
        todo!()
    }
}

/// Returns a vector aiming towards a given target from a starting position. This function does **not** account for rotational velocity, and does not support balls curving through the air.
/// 
/// This function will panic if `accuracy` is not in inclusive range `[0.0, 1.0]`.
/// 
/// # Arguments
/// * `target_pos` - the world position where the throw would land if perfectly accurate
/// * `start_pos` - the world position where the throw will originate from
/// * `throw_speed_units_per_sec` - how many in-world non-vertical units the throw will travel per second, ignoring gravity.
/// * `accuracy` - how accurate the throw is, in range `[0.0, 1.0]`, where 1.0 is perfectly accurate and 0.0 will go in a completely random direction.
fn get_throw_vector_towards_target(
    target_pos: &Vector3<f32>,
    start_pos: &Vector3<f32>,
    throw_speed_units_per_sec: f32,
    accuracy: f32,
    y_axis_gravity: f32,
) -> Vector3<f32> {
    assert!((0.0..=1.0).contains(&accuracy));

    let difference_vector = target_pos - start_pos;
    let difference_distance = difference_vector.magnitude();
    let total_travel_time_sec = difference_distance / throw_speed_units_per_sec;

    // Given that we want the ball to hit the target and our throw will be affected by gravity, we need to calculate how high to throw the ball to hit the target.
    // To do this, we'll use Newtonian falling body equations. Starting with the formula for free fall:
    //     `y_pos(t) = y_velocity(0) * t + y_pos(0) - 1/2gt^2`
    // We want to solve for `y_velocity(0)`, so we can rearrange this like the following:
    //     `y_velocity(0) = (y_pos(t) - y_pos(0)) + 1/2gt^2) / t`
    // Using the variables we have defined in this function, this maps to the following:
    //     `y_pos(t) - y_pos(0)` = `difference_vector.y`
    //     `g` = `y_axis_gravity`
    //     `t` = `total_travel_time_sec`     
    let gravity_adjustment_magnitude = (difference_vector.y + (0.5 * -y_axis_gravity * (total_travel_time_sec.powi(2)))) / total_travel_time_sec;
    
    // Our throw direction will ignore the Y direction to get a correct normal vector.
    let throw_direction = vector![difference_vector.x, 0.0, difference_vector.z].normalize();

    // Our overall throw vector is the X and Z components of the throw, and our Y component that we calculated accounting for gravity.
     

    (throw_direction * throw_speed_units_per_sec) + vector![0.0, gravity_adjustment_magnitude, 0.0]
}