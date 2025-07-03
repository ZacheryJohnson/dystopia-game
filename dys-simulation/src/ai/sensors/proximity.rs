use rapier3d::dynamics::RigidBodySet;
use rapier3d::geometry::{ColliderHandle, ColliderSet};
use rapier3d::na::Isometry3;
use rapier3d::pipeline::{QueryFilter, QueryPipeline};
use rapier3d::prelude::Cylinder;
use crate::ai::belief::{Belief, ExpiringBelief};
use crate::ai::sensor::Sensor;
use crate::game_objects::ball::BallState;
use crate::game_objects::combatant::CombatantId;
use crate::game_objects::game_object_type::GameObjectType;
use crate::game_state::{BallsMapT, CollidersMapT, CombatantsMapT};
use crate::game_tick::GameTickNumber;

#[derive(Clone, Debug)]
pub struct ProximitySensor {
    enabled: bool,
    shape: Cylinder,
    owner_combatant_id: CombatantId,
    owner_collider_handle: ColliderHandle,

    // ZJ-TODO: revisit this approach later
    yields_beliefs: bool,
}

impl ProximitySensor {
    pub fn new(
        owner_combatant_id: CombatantId,
        owner_height: f32,
        radius: f32,
        owner_collider_handle: ColliderHandle,
    ) -> ProximitySensor {
        let shape = Cylinder::new(owner_height / 2.0, radius);

        ProximitySensor {
            enabled: true,
            shape,
            owner_combatant_id,
            owner_collider_handle,
            yields_beliefs: true
        }
    }
}

impl Sensor for ProximitySensor {
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_yields_beliefs(&mut self, yields_beliefs: bool) {
        self.yields_beliefs = yields_beliefs;
    }

    fn sense(
        &self,
        combatant_isometry: &Isometry3<f32>,
        query_pipeline: &QueryPipeline,
        rigid_body_set: &RigidBodySet,
        collider_set: &ColliderSet,
        active_colliders: &CollidersMapT,
        _: &CombatantsMapT,
        balls_map: &BallsMapT,
        current_tick: GameTickNumber
    ) -> (bool, Vec<ExpiringBelief>) {
        let mut beliefs = vec![];

        let query_filter = QueryFilter::default()
            .exclude_collider(self.owner_collider_handle);

        // ZJ-TODO: this sucks please change
        let mut should_interrupt = false;

        query_pipeline.intersections_with_shape(
            rigid_body_set,
            collider_set,
            combatant_isometry,
            &self.shape,
            query_filter,
            |collider_handle| {
                if self.yields_beliefs {
                    let game_object = active_colliders.get(&collider_handle).unwrap();
                    match game_object {
                        GameObjectType::Ball(ball_id) => {
                            beliefs.push(ExpiringBelief::new(Belief::InBallPickupRange {
                                ball_id: *ball_id,
                                combatant_id: self.owner_combatant_id,
                            }, Some(current_tick + 12)));
                        },
                        GameObjectType::Combatant(combatant_id) => {
                            beliefs.push(ExpiringBelief::new(Belief::CanReachCombatant {
                                self_combatant_id: self.owner_combatant_id,
                                target_combatant_id: *combatant_id,
                            }, Some(current_tick + 1)));
                        }
                        _ => {} // we can ignore all other game object types
                    }

                    true
                } else {
                    let game_object = active_colliders.get(&collider_handle).unwrap();
                    match game_object {
                        GameObjectType::Ball(ball_id) => {
                            let ball_object = balls_map.get(ball_id).unwrap();
                            if matches!(ball_object.state, BallState::ThrownAtTarget {..}) {
                                should_interrupt = true;
                            }
                        },
                        GameObjectType::Combatant(_combatant_id) => {
                            // ZJ-TODO: determine when/if to interrupt for combatants
                        }
                        _ => {} // we can ignore all other game object types
                    }

                    true
                }

            });

        (should_interrupt, beliefs)
    }
}

