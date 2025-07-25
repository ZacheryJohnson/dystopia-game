use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use dys_world::arena::plate::PlateId;
use rapier3d::prelude::*;
use rapier3d::na::{Quaternion, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use dys_satisfiable::SatisfiableField;
use crate::ai::belief::{Belief, ExpiringBelief, SatisfiableBelief};
use crate::game_objects::{ball::BallId, combatant::CombatantId};
use crate::game_objects::ball::BallState;
use crate::game_objects::combatant::TeamAlignment;
use crate::game_objects::game_object::GameObject;
use crate::game_state::GameState;

/// Game objects can tick to generate SimulationEvents.
/// Events generated by game objects aren't guaranteed to affect the simulation however,
/// and must be wrapped as a PendingSimulationEvent.
///
/// For example, a combatant may try to throw a ball at another combatant, but be stunned
/// by a ball explosion prior to actually throwing the ball.
/// In this case, the combatant's pending action will be discarded and not committed by the simulation.
#[derive(Debug)]
pub struct PendingSimulationEvent(pub SimulationEvent);
impl Deref for PendingSimulationEvent {
    type Target = SimulationEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PendingSimulationEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PendingSimulationEvent {
    /// Returns true if both pending simulation events are the same variant of SimulationEvent
    pub fn is_same_variant(&self, other: &PendingSimulationEvent) -> bool {
        std::mem::discriminant(&self.0) == std::mem::discriminant(&other.0)
    }
}

/// SimulationEvents are any notable action that happens during a simulation.
/// These events will be collected to form a recap of the game.
/// 
/// Alongside discrete events (for example, a player was hit by a ball),
/// per-tick updates can be useful as simulation events,
/// such as the last position of a combatant or ball. These will be used in the
/// [GameLog](crate::game_log::GameLog) to allow clients to visually recreate 
/// an entire game, whereas just discrete events may be confusing to see.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SimulationEvent {
    // ZJ-TODO: keep?
    // This is currently only being used for tick zero initial state (eg where are there plates? where are there walls?)
    ArenaObjectPositionUpdate { object_type_id: u32, position: Vector3<f32>, scale: Vector3<f32>, rotation: Quaternion<f32> },

    /// A ball has moved through the world
    BallPositionUpdate {
        ball_id: BallId,
        position: Vector3<f32>,
        charge: f32,
    },

    /// A combatant has moved through the world
    CombatantPositionUpdate { combatant_id: CombatantId, position: Vector3<f32> },

    /// A combatant has begun being on a plate
    CombatantOnPlate { combatant_id: CombatantId, plate_id: PlateId },

    /// A combatant has stopped being on a plate
    CombatantOffPlate { combatant_id: CombatantId, plate_id: PlateId },

    /// A combatant has picked up a ball that was on the ground.
    CombatantPickedUpBall { combatant_id: CombatantId, ball_id: BallId },

    /// A combatant has dropped a ball without throwing it.
    CombatantDroppedBall { combatant_id: CombatantId, ball_id: BallId },

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

    ThrownBallCaught {
        thrower_id: CombatantId,
        catcher_id: CombatantId,
        ball_id: BallId,
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

    // ZJ-TODO: refactor this into a StatusEffect enum
    CombatantStunned { combatant_id: CombatantId, start: bool },

    CombatantShoveForceApplied {
        shover_combatant_id: CombatantId,
        recipient_target_id: CombatantId,
        force_magnitude: f32,
        force_direction: Vector3<f32>
    },

    // ZJ-TODO: TEMP: broadcast this belief to all other combatants (excluding self)
    BroadcastBelief {
        from_combatant_id: CombatantId,
        belief: Belief,
    }
}

impl SimulationEvent {
    pub fn simulate_event(
        game_state: Arc<Mutex<GameState>>,
        event: &PendingSimulationEvent,
    ) -> (bool, Vec<PendingSimulationEvent>) {
        match **event {
            SimulationEvent::ArenaObjectPositionUpdate { .. } => {}

            SimulationEvent::BallPositionUpdate { ball_id, position, charge: _ } => {
                let mut game_state = game_state.lock().unwrap();

                let ball_object = game_state
                    .balls
                    .get(&ball_id)
                    .unwrap()
                    .to_owned();

                // The physics sim will handle ball updates in most cases
                // However, when we're being held by a combatant, we're currently (hackily)
                // teleporting the ball around to match
                // Fix that, but until then, we need to update our transform manually
                if matches!(ball_object.state, BallState::Held {..} ) {
                    let (rigid_body_set, _) = game_state.physics_sim.sets_mut();
                    let combatant_rb = rigid_body_set
                        .get_mut(ball_object.rigid_body_handle().unwrap())
                        .unwrap();

                    combatant_rb.set_translation(position, true);
                }
            }
            SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                let mut game_state = game_state.lock().unwrap();

                let combatant_object = game_state
                    .combatants
                    .get(&combatant_id)
                    .unwrap()
                    .to_owned();

                let (rigid_body_set, _) = game_state.physics_sim.sets_mut();
                let combatant_rb = rigid_body_set
                    .get_mut(combatant_object.rigid_body_handle)
                    .unwrap();

                let old_position: &Vector3<f32> = combatant_rb.translation();
                let difference_vector = position - old_position;
                let rotation = UnitQuaternion::face_towards(&difference_vector, &Vector3::y());
                combatant_rb.set_translation(position, true);

                if rotation.axis_angle().is_some() {
                    combatant_rb.set_rotation(rotation, true);
                }
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
            SimulationEvent::CombatantPickedUpBall { combatant_id, ball_id }
            | SimulationEvent::ThrownBallCaught { thrower_id: _, catcher_id: combatant_id, ball_id }=> {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick.to_owned();

                // ZJ-TODO: be authoritative here
                //          combatants may be out of range, despite their beliefs
                let distance = {
                    let (rigid_body_set, _) = game_state.physics_sim.sets();
                    let combatant_rb_handle = game_state
                        .combatants
                        .get(&combatant_id)
                        .unwrap()
                        .rigid_body_handle()
                        .unwrap();
                    let ball_rb_handle = game_state
                        .balls
                        .get(&ball_id)
                        .unwrap()
                        .rigid_body_handle()
                        .unwrap();

                    let combatant_pos = rigid_body_set.get(combatant_rb_handle).unwrap().translation();
                    let ball_pos = rigid_body_set.get(ball_rb_handle).unwrap().translation();

                    combatant_pos - ball_pos
                };

                // ZJ-TODO: read this from combatant stats
                //          might have some long ass arms (if arms at all)
                if distance.magnitude() > 2.0 {
                    return (false, vec![]);
                }

                {
                    let combatant_object = game_state
                        .combatants
                        .get_mut(&combatant_id)
                        .unwrap();

                    // Our combatant may have been stunned since initially trying this
                    if combatant_object.is_stunned() {
                        return (false, vec![]);
                    }
                    combatant_object.pickup_ball(ball_id);
                }

                {
                    let ball_object = game_state
                        .balls
                        .get_mut(&ball_id)
                        .unwrap();
                    ball_object.set_held_by(Some(combatant_id), current_tick);
                }
            }
            SimulationEvent::CombatantDroppedBall { combatant_id, ball_id } => {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick.to_owned();

                {
                    let combatant_object = game_state
                        .combatants
                        .get_mut(&combatant_id)
                        .unwrap();

                    combatant_object.drop_ball();
                }

                {
                    let ball_object = game_state
                        .balls
                        .get_mut(&ball_id)
                        .unwrap();
                    ball_object.set_held_by(None, current_tick);
                    ball_object.change_state(current_tick, BallState::Idle);
                }
            }
            SimulationEvent::BallThrownAtEnemy { thrower_id, enemy_id: _, ball_id, ball_impulse_vector } => {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick.to_owned();

                let combatant_object = game_state
                    .combatants
                    .get_mut(&thrower_id)
                    .unwrap();

                combatant_object.drop_ball();

                let ball_object = game_state
                    .balls
                    .get_mut(&ball_id)
                    .unwrap();
                ball_object.set_held_by(None, current_tick);
                let ball_rigid_body_handle = ball_object.rigid_body_handle().unwrap();

                let (rigid_body_set, _) = game_state.physics_sim.sets_mut();
                let ball_rb = rigid_body_set.get_mut(ball_rigid_body_handle).unwrap();
                ball_rb.apply_impulse(ball_impulse_vector, true);
            }
            SimulationEvent::BallThrownAtTeammate { thrower_id, teammate_id: _, ball_id, ball_impulse_vector } => {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick.to_owned();

                let combatant_object = game_state
                    .combatants
                    .get_mut(&thrower_id)
                    .unwrap();

                combatant_object.drop_ball();

                let ball_object = game_state
                    .balls
                    .get_mut(&ball_id)
                    .unwrap();
                ball_object.set_held_by(None, current_tick);
                let ball_rigid_body_handle = ball_object.rigid_body_handle().unwrap();

                let (rigid_body_set, _) = game_state.physics_sim.sets_mut();
                let ball_rb = rigid_body_set.get_mut(ball_rigid_body_handle).unwrap();
                ball_rb.apply_impulse(ball_impulse_vector, true);
            }
            SimulationEvent::BallCollisionEnemy { .. } => {
                // ZJ-TODO: delete wholesale?
                // let mut game_state = game_state.lock().unwrap();
                // let current_tick = game_state.current_tick;
                // let ball_object = game_state.balls.get_mut(&ball_id).unwrap();
                // ball_object.change_state(current_tick, BallState::Explode);
            }
            SimulationEvent::BallCollisionArena { thrower_id: _, original_target_id: _, ball_id } => {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick;
                let ball_object = game_state.balls.get_mut(&ball_id).unwrap();
                ball_object.change_state(current_tick, BallState::Explode);
            }
            SimulationEvent::BallExplosion { ball_id, charge: _ } => {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick;
                let ball_rigid_body_handle = game_state.balls.get(&ball_id).unwrap().rigid_body_handle().unwrap();
                {
                    let (rigid_body_set, _) = game_state.physics_sim.sets_mut();

                    // After exploding, reset charge, make ball idle
                    // ZJ-TODO: delete ball, spawn new one, etc
                    let ball_rb = rigid_body_set.get_mut(ball_rigid_body_handle).unwrap();
                    ball_rb.set_linvel(vector![0.0, 0.0, 0.0], true);
                    ball_rb.set_angvel(vector![0.0, 0.0, 0.0], true);
                }

                {
                    let ball_object = game_state.balls.get_mut(&ball_id).unwrap();
                    ball_object.charge = 0.0;
                    ball_object.change_state(current_tick, BallState::Idle);
                }
            }
            SimulationEvent::BallExplosionForceApplied { ball_id: _, combatant_id, force_magnitude, force_direction } => {
                let mut game_state = game_state.lock().unwrap();
                let combatant_rigid_body_handle = game_state.combatants.get(&combatant_id).unwrap().rigid_body_handle;
                let (rigid_body_set, _) = game_state.physics_sim.sets_mut();

                let combatant_rb = rigid_body_set
                    .get_mut(combatant_rigid_body_handle)
                    .unwrap();
                let impulse = force_direction.normalize() * force_magnitude;
                combatant_rb.apply_impulse(impulse, true);

                // ZJ-TODO: apply damage to limbs, etc
                {
                    let combatant_object = game_state.combatants.get_mut(&combatant_id).unwrap();
                    combatant_object.set_stunned(true);
                    combatant_object.apply_damage(force_magnitude);
                }
            }
            SimulationEvent::PointsScoredByCombatant { plate_id: _, combatant_id, points } => {
                // ZJ-TODO: double points if no other combatants are on the plate

                let mut game_state = game_state.lock().unwrap();
                assert!(game_state.is_scoring_tick());

                let combatant_team = game_state.combatants.get_mut(&combatant_id).unwrap();

                if combatant_team.team == TeamAlignment::Home {
                    game_state.home_points += points as u16;
                } else {
                    game_state.away_points += points as u16;
                }
            }
            SimulationEvent::CombatantStunned { combatant_id, start: is_stunned } => {
                let mut game_state = game_state.lock().unwrap();

                if !is_stunned {
                    let combatant_object = game_state.combatants.get_mut(&combatant_id).unwrap();
                    combatant_object.set_stunned(false);
                    return (true, vec![]);
                }

                let current_tick = game_state.current_tick.to_owned();

                let maybe_ball_id = {
                    let combatant_object = game_state.combatants.get_mut(&combatant_id).unwrap();

                    if let Some(ball_id) = combatant_object.ball() {
                        combatant_object.drop_ball();
                        let ball_object = game_state.balls.get_mut(&ball_id).unwrap();
                        ball_object.set_held_by(None, current_tick);
                        Some(ball_id)
                    } else {
                        None
                    }
                };

                if let Some(ball_id) = maybe_ball_id {
                    let combatant_object = game_state.combatants.get_mut(&combatant_id).unwrap();
                    let mut combatant_state = combatant_object.combatant_state.lock().unwrap();
                    combatant_state.beliefs.remove_beliefs_by_test(
                        &SatisfiableBelief::HeldBall()
                            .combatant_id(SatisfiableField::Exactly(combatant_id))
                    );

                    return (true, vec![
                        PendingSimulationEvent(SimulationEvent::CombatantDroppedBall {
                            combatant_id,
                            ball_id,
                        })
                    ])
                }
            }
            SimulationEvent::CombatantShoveForceApplied { shover_combatant_id: _, recipient_target_id, force_magnitude, force_direction } => {
                let mut game_state = game_state.lock().unwrap();

                let combatant_rigid_body_handle = {
                    let combatant_object = game_state.combatants.get_mut(&recipient_target_id).unwrap();
                    combatant_object.set_stunned(true);
                    combatant_object.apply_damage(force_magnitude / 2.0); // arbitrarily making shoves do less damage
                    combatant_object.rigid_body_handle
                };

                let (rigid_body_set, _) = game_state.physics_sim.sets_mut();

                let combatant_rb = rigid_body_set
                    .get_mut(combatant_rigid_body_handle)
                    .unwrap();
                let impulse = force_direction.normalize() * force_magnitude;
                combatant_rb.apply_impulse(impulse, true);
            },

            SimulationEvent::BroadcastBelief {from_combatant_id, belief} => {
                let mut game_state = game_state.lock().unwrap();
                let current_tick = game_state.current_tick.to_owned();
                for (combatant_id, combatant_object) in game_state.combatants.iter_mut() {
                    if *combatant_id == from_combatant_id {
                        // Don't broadcast to ourselves
                        continue;
                    }

                    // ZJ-TODO: will unsourced belief be a problem?
                    let mut combatant_state = combatant_object.combatant_state.lock().unwrap();
                    combatant_state.beliefs.add_expiring_belief(
                        ExpiringBelief::new(belief, Some(current_tick + 12))
                    );
                }
            }
        };

        (true, vec![])
    }
}