use std::sync::{Arc, Mutex};
use rand_distr::num_traits::Zero;
use dys_world::arena::plate::PlateId;
use rapier3d::prelude::*;
use rapier3d::na::{Quaternion, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use crate::ai::belief::Belief;
use crate::game_objects::{ball::BallId, combatant::CombatantId};
use crate::game_objects::ball::BallState;
use crate::game_objects::game_object::GameObject;
use crate::game_state::GameState;

pub struct PendingSimulationTick {
    simulation_event: SimulationEvent,
    beliefs_upon_confirmation: Vec<Belief>,
}

/// SimulationEvents are any notable action that happens during a simulation.
/// These events will be collected to form a recap of the game.
/// 
/// Alongside discrete events (for example, a player was hit by a ball),
/// per-tick updates can be useful as simulation events,
/// such as the last position of a combatant or ball. These will be used in the
/// [GameLog](crate::game_log::GameLog) to allow clients to visually recreate 
/// an entire game, whereas just discrete events may be confusing to see.
#[derive(Debug, Deserialize, Serialize)]
pub enum SimulationEvent {
    // ZJ-TODO: keep?
    // This is currently only being used for tick zero initial state (eg where are there plates? where are there walls?)
    ArenaObjectPositionUpdate { object_type_id: u32, position: Vector3<f32>, scale: Vector3<f32>, rotation: Quaternion<f32> },

    /// A ball has moved through the world
    BallPositionUpdate { ball_id: BallId, position: Vector3<f32> },

    /// A combatant has moved through the world
    CombatantPositionUpdate { combatant_id: CombatantId, position: Vector3<f32> },

    /// A combatant has begun being on a plate
    CombatantOnPlate { combatant_id: CombatantId, plate_id: PlateId },

    /// A combatant has stopped being on a plate
    CombatantOffPlate { combatant_id: CombatantId, plate_id: PlateId },

    /// A combatant has picked up a ball that was on the ground.
    CombatantPickedUpBall { combatant_id: CombatantId, ball_id: BallId },

    /// A ball has been thrown targeting an enemy
    BallThrownAtEnemy {
        thrower_id: CombatantId,
        enemy_id: CombatantId,
        ball_id: BallId,
        ball_impulse_vector: Vector3<f32>,
    },

    /// A ball has been thrown targeting a teammate
    BallThrownAtTeammate {
        thrower_id: CombatantId,
        teammate_id: CombatantId,
        ball_id: BallId,
        ball_impulse_vector: Vector3<f32>,
    },

    /// A ball has collided with an enemy 
    BallCollisionEnemy { thrower_id: CombatantId, enemy_id: CombatantId, ball_id: BallId },

    /// A ball has collided with the ground, defusing it
    BallCollisionArena { thrower_id: CombatantId, original_target_id: CombatantId, ball_id: BallId },

    /// A ball has exploded
    BallExplosion { ball_id: BallId, charge: f32 },

    /// A ball explosion has applied explosion force to a combatant
    BallExplosionForceApplied { ball_id: BallId, combatant_id: CombatantId, force_magnitude: f32, force_direction: Vector3<f32> },

    /// Points have been scored this tick by a combatant on a plate
    PointsScoredByCombatant { plate_id: PlateId, combatant_id: CombatantId, points: u8 },
}

impl SimulationEvent {
    pub fn simulate_event(
        game_state: Arc<Mutex<GameState>>,
        event: &SimulationEvent,
    ) -> bool {
        match *event {
            SimulationEvent::ArenaObjectPositionUpdate { .. } => {}

            // handled by physics
            SimulationEvent::BallPositionUpdate { .. } => {}

            SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                let mut game_state = game_state.lock().unwrap();

                let combatant_object = game_state
                    .combatants
                    .get(&combatant_id)
                    .unwrap()
                    .to_owned();

                let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();
                let combatant_rb = rigid_body_set
                    .get_mut(combatant_object.rigid_body_handle)
                    .unwrap();

                let old_position: &Vector3<f32> = combatant_rb.translation();
                let difference_vector = (position - old_position);
                let rotation = UnitQuaternion::face_towards(&difference_vector, &Vector3::y());
                combatant_rb.set_translation(position, true);

                if rotation.axis_angle().is_some() {
                    combatant_rb.set_rotation(rotation, true);
                }

                // ZJ-TODO: investigate if using kinematic controllers would be better
                // combatant_rb.set_next_kinematic_translation(new_combatant_position);
            }
            SimulationEvent::CombatantOnPlate { combatant_id, plate_id } => {
                let mut game_state = game_state.lock().unwrap();
                let combatant_object = game_state
                    .combatants
                    .get_mut(&combatant_id)
                    .unwrap();

                combatant_object.set_on_plate(plate_id);
            }
            SimulationEvent::CombatantOffPlate { combatant_id, plate_id: _ } => {
                let mut game_state = game_state.lock().unwrap();
                let combatant_object = game_state
                    .combatants
                    .get_mut(&combatant_id)
                    .unwrap();

                combatant_object.set_off_plate();
            }
            SimulationEvent::CombatantPickedUpBall { combatant_id, ball_id } => {
                let mut game_state = game_state.lock().unwrap();

                {
                    let combatant_object = game_state
                        .combatants
                        .get_mut(&combatant_id)
                        .unwrap();

                    combatant_object.pickup_ball(ball_id);
                }

                {
                    let current_tick = game_state.current_tick;
                    let ball_object = game_state
                        .balls
                        .get_mut(&ball_id)
                        .unwrap();
                    ball_object.set_held_by(Some(combatant_id), current_tick);
                }
            }
            SimulationEvent::BallThrownAtEnemy { thrower_id, enemy_id: _, ball_id, ball_impulse_vector } => {
                let mut game_state = game_state.lock().unwrap();
                let combatant_object = game_state
                    .combatants
                    .get_mut(&thrower_id)
                    .unwrap();

                combatant_object.drop_ball();

                let ball_rigid_body_handle = {
                    let current_tick = game_state.current_tick;
                    let mut ball_object = game_state.balls.get_mut(&ball_id).unwrap();
                    ball_object.set_held_by(None, current_tick);

                    ball_object.rigid_body_handle().unwrap()
                };

                let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();
                let ball_rb = rigid_body_set.get_mut(ball_rigid_body_handle).unwrap();
                ball_rb.apply_impulse(ball_impulse_vector, true);
            }
            SimulationEvent::BallThrownAtTeammate { thrower_id, teammate_id, ball_id, ball_impulse_vector } => {
                let mut game_state = game_state.lock().unwrap();
                let combatant_object = game_state
                    .combatants
                    .get_mut(&thrower_id)
                    .unwrap();

                combatant_object.drop_ball();

                let ball_rigid_body_handle = {
                    let current_tick = game_state.current_tick;
                    let mut ball_object = game_state.balls.get_mut(&ball_id).unwrap();
                    ball_object.set_held_by(None, current_tick);

                    ball_object.rigid_body_handle().unwrap()
                };

                let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();
                let ball_rb = rigid_body_set.get_mut(ball_rigid_body_handle).unwrap();
                ball_rb.apply_impulse(ball_impulse_vector, true);
            }
            SimulationEvent::BallCollisionEnemy { ball_id, .. } => {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick;
                let ball_object = game_state.balls.get_mut(&ball_id).unwrap();
                ball_object.change_state(current_tick, BallState::Explode);
            }
            SimulationEvent::BallCollisionArena { .. } => {
                // ZJ-TODO: 'disable' ball
            }
            SimulationEvent::BallExplosion { ball_id, charge } => {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick;
                let ball_rigid_body_handle = game_state.balls.get(&ball_id).unwrap().rigid_body_handle().unwrap();
                {
                    let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();

                    // After exploding, reset charge, make ball idle
                    // ZJ-TODO: delete ball, spawn new one, etc
                    let ball_rb = rigid_body_set.get_mut(ball_rigid_body_handle).unwrap();
                    ball_rb.set_linvel(vector![0.0, 0.0, 0.0], true);
                    ball_rb.set_angvel(vector![0.0, 0.0, 0.0], true);
                }

                {
                    let mut ball_object = game_state.balls.get_mut(&ball_id).unwrap();
                    ball_object.charge = 0.0;
                    ball_object.change_state(current_tick, BallState::Idle);
                }
            }
            SimulationEvent::BallExplosionForceApplied { ball_id, combatant_id, force_magnitude, force_direction } => {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick;
                let combatant_rigid_body_handle = game_state.combatants.get(&combatant_id).unwrap().rigid_body_handle;
                let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();

                // ZJ_TODO: investigate kinematic rigid bodies

                let combatant_rb = rigid_body_set
                    .get_mut(combatant_rigid_body_handle)
                    .unwrap();
                let impulse = force_direction.normalize() * force_magnitude;
                combatant_rb.apply_impulse(impulse, true);

                // ZJ-TODO: apply damage to limbs, etc
                {
                    let combatant_object = game_state.combatants.get_mut(&combatant_id).unwrap();
                    let mut combatant_state = combatant_object.combatant_state.lock().unwrap();
                    combatant_state.stunned_by_explosion = true;
                }
            }
            SimulationEvent::PointsScoredByCombatant { .. } => {}
        };

        true
    }
}