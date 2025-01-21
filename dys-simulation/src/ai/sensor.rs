use std::collections::HashMap;
use rapier3d::geometry::ColliderHandle;
use rapier3d::na::Isometry3;
use rapier3d::prelude::{ColliderSet, Cone, QueryFilter, QueryPipeline, RigidBodySet, SharedShape};
use crate::ai::belief::Belief;
use crate::game_objects::game_object::GameObject;
use crate::game_objects::game_object_type::GameObjectType;
use crate::game_state::{BallsMapT, CombatantsMapT};

pub trait Sensor {
    fn set_enabled(&mut self, enabled: bool);
    fn enabled(&self) -> bool;
    fn sense(
        &self,
        combatant_isometry: &Isometry3<f32>,
        query_pipeline: &QueryPipeline,
        rigid_body_set: &RigidBodySet,
        collider_set: &ColliderSet,
        active_colliders: &HashMap<ColliderHandle, GameObjectType>,
        combatants: &CombatantsMapT,
        balls: &BallsMapT,
    ) -> Vec<Belief>;
}

#[derive(Clone, Debug)]
pub struct LineOfSightSensor {
    enabled: bool,
    shape: Cone,
}

impl LineOfSightSensor {
    pub fn new(
        sight_distance: f32,
        fov_angle: f32,
    ) -> LineOfSightSensor {
        let half_height = sight_distance / 2.0;
        let radius = sight_distance * (fov_angle.to_radians() / 2.0).tan();
        let shape = Cone::new(half_height, radius);

        LineOfSightSensor {
            enabled: true,
            shape,
        }
    }
}

impl Sensor for LineOfSightSensor {
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn sense(
        &self,
        combatant_isometry: &Isometry3<f32>,
        query_pipeline: &QueryPipeline,
        rigid_body_set: &RigidBodySet,
        collider_set: &ColliderSet,
        active_colliders: &HashMap<ColliderHandle, GameObjectType>,
        combatants: &CombatantsMapT,
        balls: &BallsMapT,
    ) -> Vec<Belief> {
        let mut beliefs = vec![];

        query_pipeline.intersections_with_shape(
            rigid_body_set,
            collider_set,
            &combatant_isometry,
            &self.shape,
            QueryFilter::default(),
            |collider_handle| {
                let game_object = active_colliders.get(&collider_handle).unwrap();
                match game_object {
                    GameObjectType::Ball(ball_id) => {
                        let ball_object = balls.get(ball_id).unwrap();
                        let ball_pos = rigid_body_set
                            .get(ball_object.rigid_body_handle().unwrap())
                            .unwrap()
                            .translation();

                        beliefs.push(Belief::BallPosition {
                            ball_id: *ball_id,
                            position: ball_pos.to_owned(),
                        });

                        if let Some(combatant_id) = ball_object.held_by {
                            beliefs.push(Belief::HeldBall {
                                ball_id: *ball_id,
                                combatant_id,
                            });
                        }
                    },
                    GameObjectType::Combatant(combatant_id) => {
                        let combatant_object = combatants.get(combatant_id).unwrap();
                        let combatant_pos = rigid_body_set
                            .get(combatant_object.rigid_body_handle().unwrap())
                            .unwrap()
                            .translation();

                        beliefs.push(Belief::CombatantPosition {
                            combatant_id: *combatant_id,
                            position: combatant_pos.to_owned(),
                        });
                    },
                    _ => {} // we can ignore all other game object types
                }

                true
            });

        beliefs
    }
}