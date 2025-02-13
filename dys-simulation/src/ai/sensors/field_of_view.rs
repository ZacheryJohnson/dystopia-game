use rapier3d::dynamics::RigidBodySet;
use rapier3d::geometry::{ColliderHandle, ColliderSet, Cuboid};
use rapier3d::prelude::*;
use rapier3d::na::{vector, Isometry3};
use rapier3d::pipeline::{QueryFilter, QueryPipeline};
use crate::ai::belief::Belief;
use crate::ai::sensor::Sensor;
use crate::game_objects::ball::BallState;
use crate::game_objects::game_object::GameObject;
use crate::game_objects::game_object_type::GameObjectType;
use crate::game_state::{BallsMapT, CollidersMapT, CombatantsMapT};

#[derive(Clone, Debug)]
pub struct FieldOfViewSensor {
    enabled: bool,
    shape: Cuboid,
    isometry_offset: Isometry3<f32>,
    owner_collider_handle: ColliderHandle,
}

impl FieldOfViewSensor {
    pub fn new(
        sight_distance: f32,
        owner_collider_handle: ColliderHandle,
    ) -> FieldOfViewSensor {
        let half_dist = sight_distance / 2.0;
        // ZJ-TODO: ideally this would be a cone, not a cuboid, but I can't get the cone to work
        //          once the game is actually fun and playable, possibly revisit
        let shape = Cuboid::new(vector![half_dist, 5.0, half_dist]);
        let isometry_offset = Isometry3::translation(0.0, 0.0, half_dist);

        FieldOfViewSensor {
            enabled: true,
            shape,
            isometry_offset,
            owner_collider_handle,
        }
    }
}

impl Sensor for FieldOfViewSensor {
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
        active_colliders: &CollidersMapT,
        combatants: &CombatantsMapT,
        balls: &BallsMapT,
    ) -> Vec<Belief> {
        let mut beliefs = vec![];

        let query_filter = QueryFilter::default()
            .exclude_collider(self.owner_collider_handle);

        let mut new_isometry = self.isometry_offset.to_owned();
        new_isometry.append_rotation_mut(&combatant_isometry.rotation);
        new_isometry.append_translation_mut(&combatant_isometry.translation);

        query_pipeline.intersections_with_shape(
            rigid_body_set,
            collider_set,
            &new_isometry,
            &self.shape,
            query_filter,
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

                        if matches!(ball_object.state, BallState::ThrownAtTarget {..}) {
                            beliefs.push(Belief::BallIsFlying { ball_id: *ball_id });
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

                        if let Some(ball_id) = combatant_object.ball() {
                            beliefs.push(Belief::HeldBall {
                                combatant_id: *combatant_id,
                                ball_id,
                            })
                        }
                    },
                    _ => {} // we can ignore all other game object types
                }

                true
            });

        beliefs
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::StdRng;
    use rand::SeedableRng;
    use rapier3d::na::vector;
    use rapier3d::prelude::*;
    use GameObjectType::Combatant;
    use crate::ai::belief::Belief;
    use crate::ai::sensor::Sensor;
    use crate::ai::sensors::field_of_view::FieldOfViewSensor;
    use crate::game_objects::combatant::{CombatantObject, TeamAlignment};
    use crate::game_objects::game_object_type::GameObjectType;
    use crate::game_state::{BallsMapT, CollidersMapT, CombatantsMapT};
    use crate::generator::Generator;
    use crate::physics_sim::PhysicsSim;

    #[test]
    fn should_see_combatant_directly_in_front() {
        let world = Generator::new().generate_world(&mut StdRng::from_entropy());
        let (combatant_1_instance, combatant_2_instance, combatant_3_instance) = {
            (world.combatants[0].clone(), world.combatants[1].clone(), world.combatants[2].clone())
        };

        // Combatant 1 is who the sensor will be "attached" to
        let combatant_1_position = vector![1.0, 0.0, 0.0];
        // Combatant 2 is in front of combatant 1
        let combatant_2_position = vector![1.0, 0.0, 3.0];
        // Combatant 3 is behind combatant 1
        let combatant_3_position = vector![1.0, 0.0, -3.0];

        let mut physics_sim = PhysicsSim::new(10);
        let (combatant_1_collider_handle, combatant_1, active_colliders, combatants) = {
            let (
                rigid_body_set,
                collider_set,
                _,
            ) = physics_sim.sets_mut();

            let combatant_1 = CombatantObject::new(
                1,
                combatant_1_instance,
                combatant_1_position.clone(),
                TeamAlignment::Home,
                rigid_body_set,
                collider_set,
            );

            let combatant_2 = CombatantObject::new(
                2,
                combatant_2_instance,
                combatant_2_position.clone(),
                TeamAlignment::Home,
                rigid_body_set,
                collider_set,
            );

            let combatant_3 = CombatantObject::new(
                3,
                combatant_3_instance,
                combatant_3_position.clone_owned(),
                TeamAlignment::Away,
                rigid_body_set,
                collider_set,
            );

            // We must tick in order for the objects to be available in our tests
            physics_sim.tick();

            let mut active_colliders = CollidersMapT::new();
            let combatant_1_collider_handle = combatant_1.collider_handle.clone();
            active_colliders.insert(combatant_1.collider_handle, Combatant(combatant_1.id));
            active_colliders.insert(combatant_2.collider_handle, Combatant(combatant_2.id));
            active_colliders.insert(combatant_3.collider_handle, Combatant(combatant_3.id));

            let mut combatants = CombatantsMapT::new();
            combatants.insert(combatant_1.id, combatant_1.clone());
            combatants.insert(combatant_2.id, combatant_2);
            combatants.insert(combatant_3.id, combatant_3);

            (combatant_1_collider_handle, combatant_1, active_colliders, combatants)
        };

        {
            let (
                query_pipeline,
                rigid_body_set,
                collider_set,
            ) = physics_sim.query_pipeline_and_sets();

            let field_of_view_sensor = FieldOfViewSensor::new(
                10.0,
                combatant_1_collider_handle,
            );

            let combatant_forward_isometry = combatant_1.forward_isometry(rigid_body_set);

            let new_beliefs = field_of_view_sensor.sense(
                &combatant_forward_isometry,
                query_pipeline,
                rigid_body_set,
                collider_set,
                &active_colliders,
                &combatants,
                &BallsMapT::default()
            );

            assert_eq!(new_beliefs.len(), 1);

            // We should only be aware of combatant 2
            let new_belief = new_beliefs.first().unwrap();
            assert!(matches!(new_belief, Belief::CombatantPosition { combatant_id: 2, .. }));
        }
        {
            let (rigid_body_set, _, _) = physics_sim.sets_mut();
            // Rotate combatant 1 around to face combatant 3
            let combatant_1_rigid_body = rigid_body_set
                .get_mut(combatant_1.rigid_body_handle)
                .unwrap();

            combatant_1_rigid_body.set_rotation(Rotation::from_scaled_axis(
                vector![0.0, 180.0_f32.to_radians(), 0.0]),
                                                true
            );
        }

        physics_sim.tick();

        {
            let (
                query_pipeline,
                rigid_body_set,
                collider_set,
            ) = physics_sim.query_pipeline_and_sets();

            let field_of_view_sensor = FieldOfViewSensor::new(
                10.0,
                combatant_1_collider_handle,
            );

            let combatant_forward_isometry = combatant_1.forward_isometry(rigid_body_set);

            let new_beliefs = field_of_view_sensor.sense(
                &combatant_forward_isometry,
                query_pipeline,
                rigid_body_set,
                collider_set,
                &active_colliders,
                &combatants,
                &BallsMapT::default()
            );

            assert_eq!(new_beliefs.len(), 1);

            // We should only be aware of combatant 3
            let new_belief = new_beliefs.first().unwrap();
            assert!(matches!(new_belief, Belief::CombatantPosition { combatant_id: 3, .. }));
        }

    }
}