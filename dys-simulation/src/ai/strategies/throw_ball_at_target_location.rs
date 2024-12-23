use std::sync::{Arc, Mutex};
use rapier3d::{na::Vector3, prelude::*};

use crate::{ai::{agent::Agent, belief::Belief, strategy::Strategy}, game_objects::{combatant::CombatantId, game_object::GameObject}, game_state::{GameState}, simulation::simulation_event::SimulationEvent};

pub struct ThrowBallAtTargetStrategy {
    target: CombatantId,
    is_complete: bool,
}

impl ThrowBallAtTargetStrategy {
    pub fn new(target_combatant: CombatantId) -> ThrowBallAtTargetStrategy {
        ThrowBallAtTargetStrategy {
            target: target_combatant,
            is_complete: false,
        }
    }
}

impl Strategy for ThrowBallAtTargetStrategy {
    fn name(&self) -> String {
        String::from("Throw Ball at Target")
    }

    fn can_perform(&self, owned_beliefs: &[Belief]) -> bool {
        owned_beliefs.contains(&Belief::SelfHasBall)
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn tick(
        &mut self,
        agent: &dyn Agent,
        game_state: Arc<Mutex<GameState>>,
    ) -> Option<Vec<SimulationEvent>> {        
        // Agents may believe that they're holding a ball, but not actually holding a ball per the simulation
        // If the authoritative game state says they're not holding a ball, consider this strategy complete
        // ZJ-TODO: delay first?
        let Some(ball_id) = agent.combatant().combatant_state.holding_ball else {
            self.is_complete = true;
            return None;
        };

        let (target_pos, ball_pos, is_same_team, y_axis_gravity) = {
            let game_state = game_state.lock().unwrap();

            let (rigid_body_set, _, _) = game_state.physics_sim.sets();

            let target_object = game_state.combatants.get(&self.target).unwrap();
            let target_pos = rigid_body_set
                .get(target_object.rigid_body_handle)
                .unwrap()
                .translation()
                .to_owned();

            let ball_object = game_state.balls.get(&ball_id).unwrap();
            let ball_pos = rigid_body_set
                .get(ball_object.rigid_body_handle().unwrap())
                .unwrap()
                .translation()
                .to_owned();

            let is_same_team = agent.combatant().team == target_object.team;
            let y_axis_gravity = game_state.physics_sim.gravity_y();

            (target_pos, ball_pos, is_same_team, y_axis_gravity)
        };


        let throw_speed_units_per_sec_hack = 30.0_f32;
        let accuracy_hack = 1.0_f32;

        let ball_impulse_vector = get_throw_vector_towards_target(
            &target_pos,
            &ball_pos,
            throw_speed_units_per_sec_hack,
            accuracy_hack,
            y_axis_gravity
        );

        // ZJ-TODO: wait for some delay to simulate a "windup" for a throw - should we allow movement in a direction (eg crow hop)?

        // ZJ-TODO: move these to simulation processing
        // ball_rb.apply_impulse(ball_impulse, true);
        // agent.combatant_mut().combatant_state.holding_ball = None;

        Some(vec![
            if is_same_team {
                SimulationEvent::BallThrownAtTeammate { thrower_id: agent.combatant().id, teammate_id: self.target, ball_id, ball_impulse_vector }
            } else {
                SimulationEvent::BallThrownAtEnemy { thrower_id: agent.combatant().id, enemy_id: self.target, ball_id, ball_impulse_vector }
            }
        ])
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