use std::sync::{Arc, Mutex};
use dys_world::arena::plate::PlateId;
use rapier3d::na::{Quaternion, Vector3};
use serde::{Deserialize, Serialize};

use crate::game_objects::{ball::BallId, combatant::CombatantId};
use crate::game_state::GameState;

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

                combatant_rb.set_translation(position, true);

                // ZJ-TODO: investigate if using kinematic controllers would be better
                // combatant_rb.set_next_kinematic_translation(new_combatant_position);
            }
            SimulationEvent::CombatantOnPlate { combatant_id, plate_id } => {
                let mut game_state = game_state.lock().unwrap();
                let mut combatant_object = game_state
                    .combatants
                    .get_mut(&combatant_id)
                    .unwrap();

                combatant_object.set_on_plate(plate_id);
            }
            SimulationEvent::CombatantOffPlate { combatant_id, plate_id: _ } => {
                let mut game_state = game_state.lock().unwrap();
                let mut combatant_object = game_state
                    .combatants
                    .get_mut(&combatant_id)
                    .unwrap();

                combatant_object.set_off_plate();
            }
            SimulationEvent::CombatantPickedUpBall { combatant_id, ball_id } => {
                let mut game_state = game_state.lock().unwrap();
                let mut combatant_object = game_state
                    .combatants
                    .get_mut(&combatant_id)
                    .unwrap();

                combatant_object.pickup_ball(ball_id);
            }
            SimulationEvent::BallThrownAtEnemy { thrower_id, enemy_id: _, ball_id: _, ball_impulse_vector: _ } => {
                let mut game_state = game_state.lock().unwrap();
                let mut combatant_object = game_state
                    .combatants
                    .get_mut(&thrower_id)
                    .unwrap();

                combatant_object.drop_ball();
            }
            SimulationEvent::BallThrownAtTeammate { thrower_id, teammate_id: _, ball_id: _, ball_impulse_vector: _ } => {
                let mut game_state = game_state.lock().unwrap();
                let mut combatant_object = game_state
                    .combatants
                    .get_mut(&thrower_id)
                    .unwrap();

                combatant_object.drop_ball();
            }
            SimulationEvent::BallCollisionEnemy { thrower_id, enemy_id, ball_id } => {
                // ZJ-TODO: explosion logic
            }
            SimulationEvent::BallCollisionArena { thrower_id, original_target_id, ball_id } => {}
            SimulationEvent::BallExplosion { .. } => {}
            SimulationEvent::BallExplosionForceApplied { .. } => {}
            SimulationEvent::PointsScoredByCombatant { .. } => {}
        };

        true
    }
}