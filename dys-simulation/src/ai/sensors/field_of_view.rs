use rapier3d::dynamics::RigidBodySet;
use rapier3d::geometry::{ColliderHandle, ColliderSet, Cuboid};
use rapier3d::prelude::*;
use rapier3d::na::{vector, Isometry3, Vector3};
use rapier3d::pipeline::{QueryFilter, QueryPipeline};
use crate::ai::belief::{Belief, ExpiringBelief};
use crate::ai::sensor::Sensor;
use crate::game_objects::ball::BallState;
use crate::game_objects::combatant::CombatantId;
use crate::game_objects::game_object::GameObject;
use crate::game_objects::game_object_type::GameObjectType;
use crate::game_state::{BallsMapT, CollidersMapT, CombatantsMapT};
use crate::game_tick::GameTickNumber;

#[derive(Clone, Debug)]
pub struct FieldOfViewSensor {
    enabled: bool,
    shape: Cuboid,
    isometry_offset: Isometry3<f32>,
    owner_combatant_id: CombatantId,
    owner_collider_handle: ColliderHandle,
}

impl FieldOfViewSensor {
    pub fn new(
        sight_distance: f32,
        owner_combatant_id: CombatantId,
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
            owner_combatant_id,
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
        current_tick: GameTickNumber,
    ) -> (bool, Vec<ExpiringBelief>) {
        let mut beliefs = vec![];

        let shape_query_filter = QueryFilter::default()
            .exclude_collider(self.owner_collider_handle);

        let mut new_isometry = self.isometry_offset.to_owned();
        new_isometry.append_rotation_mut(&combatant_isometry.rotation);
        new_isometry.append_translation_mut(&combatant_isometry.translation);

        let mut rays_to_cast: Vec<Vector3<f32>> = Vec::new();
        query_pipeline.intersections_with_shape(
            rigid_body_set,
            collider_set,
            &new_isometry,
            &self.shape,
            shape_query_filter,
            |collider_handle| {
                let game_object = active_colliders.get(&collider_handle).unwrap();
                match game_object {
                    GameObjectType::Combatant(_) | GameObjectType::Ball(_) => {
                        let collision_pos = collider_set
                            .get(collider_handle)
                            .unwrap()
                            .translation();

                        let difference_vector = collision_pos - combatant_isometry.translation.vector;

                        rays_to_cast.push(difference_vector);
                    },
                    _ => {},
                }

                true
            }
        );

        for ray_to_cast in &rays_to_cast {
            let mut ray_collisions: Vec<(ColliderHandle, f32)> = vec![];
            let ray = Ray::new(combatant_isometry.translation.vector.into(), ray_to_cast.to_owned());
            query_pipeline.intersections_with_ray(
                rigid_body_set,
                collider_set,
                &ray,
                ray_to_cast.magnitude(),
                false,
                shape_query_filter,
                |collider_handle, ray_intersection| {
                    let collision_point = ray.point_at(ray_intersection.time_of_impact).coords;
                    let difference_vector = collision_point - combatant_isometry.translation.vector;
                    let distance = difference_vector.magnitude();

                    ray_collisions.push((collider_handle, distance));

                    true
                }
            );

            ray_collisions.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

            // ZJ-TODO: HACK: we currently see things "through" walls.
            //          This is not a strictly bad thing, as otherwise combatants would have
            //          no idea about balls behind walls.
            //          The below prevents that, BUT has the side effect of combatants forgetting
            //          about balls behind walls, as they have goldfish like memory.
            let mut has_direct_line_of_sight = true;
            for (collider_handle, _) in ray_collisions {
                let game_object = active_colliders.get(&collider_handle).unwrap();
                match game_object {
                    GameObjectType::Barrier => {
                        has_direct_line_of_sight = false;
                    },
                    GameObjectType::Ball(ball_id) => {
                        let ball_object = balls.get(ball_id).unwrap();
                        let ball_rb = rigid_body_set.get(ball_object.rigid_body_handle().unwrap()).unwrap();
                        let ball_pos = ball_rb.translation();

                        beliefs.push(ExpiringBelief::new(Belief::BallPosition {
                            ball_id: *ball_id,
                            position: ball_pos.to_owned(),
                            trajectory: ball_rb.linvel().data.0.into(),
                        }, Some(current_tick + 12)));

                        if let Some(combatant_id) = ball_object.held_by {
                            beliefs.push(ExpiringBelief::new(Belief::HeldBall {
                                ball_id: *ball_id,
                                combatant_id,
                            }, Some(current_tick + 4)));
                        }

                        if matches!(ball_object.state, BallState::ThrownAtTarget {..}) {
                            beliefs.push(ExpiringBelief::new(
                                Belief::BallIsFlying { ball_id: *ball_id },
                                Some(current_tick + 4),
                            ));
                        }
                    },
                    GameObjectType::Combatant(combatant_id) => {
                        let combatant_object = combatants.get(combatant_id).unwrap();
                        let combatant_pos = rigid_body_set
                            .get(combatant_object.rigid_body_handle().unwrap())
                            .unwrap()
                            .translation();

                        beliefs.push(ExpiringBelief::new(Belief::CombatantPosition {
                            combatant_id: *combatant_id,
                            position: combatant_pos.to_owned(),
                        }, Some(current_tick + 12)));

                        if let Some(ball_id) = combatant_object.ball() {
                            beliefs.push(ExpiringBelief::new(Belief::HeldBall {
                                combatant_id: *combatant_id,
                                ball_id,
                            }, Some(current_tick + 1)));
                        }

                        if let Some(plate_id) = combatant_object.plate() {
                            beliefs.push(ExpiringBelief::new(Belief::OnPlate {
                                combatant_id: *combatant_id,
                                plate_id,
                            }, Some(current_tick + 1)));
                        }

                        if has_direct_line_of_sight {
                            beliefs.push(ExpiringBelief::new(
                                Belief::DirectLineOfSightToCombatant {
                                    self_combatant_id: self.owner_combatant_id,
                                    other_combatant_id: *combatant_id,
                                },
                                Some(current_tick + 1),
                            ));
                        }
                    },
                    _ => {} // we can ignore all other game object types
                }
            }
        }

        (false, beliefs)
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::StdRng;
    use rand::SeedableRng;
    use rand_distr::num_traits::Zero;
    use rapier3d::na::{vector, Vector3};
    use rapier3d::prelude::*;
    use dys_satisfiable::{SatisfiabilityTest, SatisfiableField};
    use dys_world::arena::barrier::{ArenaBarrier, BarrierPathing};
    use dys_world::arena::feature::ArenaFeature;
    use dys_world::generator::Generator;
    use GameObjectType::Combatant;
    use crate::ai::belief::SatisfiableBelief;
    use crate::ai::sensor::Sensor;
    use crate::ai::sensors::field_of_view::FieldOfViewSensor;
    use crate::game_objects::combatant::{CombatantObject, TeamAlignment};
    use crate::game_objects::game_object_type::GameObjectType;
    use crate::game_objects::game_object_type::GameObjectType::Barrier;
    use crate::game_state::{BallsMapT, CollidersMapT, CombatantsMapT};
    use crate::physics_sim::PhysicsSim;

    #[test]
    fn should_see_combatant_directly_in_front() {
        let world = Generator::new().generate_world(&mut StdRng::from_os_rng());
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
                Vector3::zero(),
                TeamAlignment::Home,
                rigid_body_set,
                collider_set,
            );

            let combatant_2 = CombatantObject::new(
                2,
                combatant_2_instance,
                combatant_2_position.clone(),
                Vector3::zero(),
                TeamAlignment::Home,
                rigid_body_set,
                collider_set,
            );

            let combatant_3 = CombatantObject::new(
                3,
                combatant_3_instance,
                combatant_3_position.clone_owned(),
                Vector3::zero(),
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
                1,
                combatant_1_collider_handle,
            );

            let combatant_forward_isometry = combatant_1.forward_isometry(rigid_body_set);

            let (_, new_beliefs) = field_of_view_sensor.sense(
                &combatant_forward_isometry,
                query_pipeline,
                rigid_body_set,
                collider_set,
                &active_colliders,
                &combatants,
                &BallsMapT::default(),
                1
            );

            let knows_combatant_2_position = new_beliefs.iter().any(|belief| {
                SatisfiableBelief::CombatantPosition()
                    .combatant_id(SatisfiableField::Exactly(2))
                    .satisfied_by(belief.belief.to_owned())
            });

            let knows_no_other_positions = !new_beliefs.iter().any(|belief| {
                SatisfiableBelief::CombatantPosition()
                    .combatant_id(SatisfiableField::NotExactly(2))
                    .satisfied_by(belief.belief.to_owned())
            });

            assert!(knows_combatant_2_position && knows_no_other_positions);
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
                1,
                combatant_1_collider_handle,
            );

            let combatant_forward_isometry = combatant_1.forward_isometry(rigid_body_set);

            let (_, new_beliefs) = field_of_view_sensor.sense(
                &combatant_forward_isometry,
                query_pipeline,
                rigid_body_set,
                collider_set,
                &active_colliders,
                &combatants,
                &BallsMapT::default(),
                1
            );

            let knows_combatant_3_position = new_beliefs.iter().any(|belief| {
                SatisfiableBelief::CombatantPosition()
                    .combatant_id(SatisfiableField::Exactly(3))
                    .satisfied_by(belief.belief.to_owned())
            });

            let knows_no_other_positions = !new_beliefs.iter().any(|belief| {
                SatisfiableBelief::CombatantPosition()
                    .combatant_id(SatisfiableField::NotExactly(3))
                    .satisfied_by(belief.belief.to_owned())
            });

            assert!(knows_combatant_3_position && knows_no_other_positions);
        }
    }

    #[test]
    fn should_not_see_through_walls() {
        let world = Generator::new().generate_world(&mut StdRng::from_os_rng());
        let (combatant_1_instance, combatant_2_instance) = {
            (world.combatants[0].clone(), world.combatants[1].clone())
        };

        // Combatant 1 is who the sensor will be "attached" to
        let combatant_1_position = vector![1.0, 0.0, 0.0];
        // Combatant 2 is in front of combatant 1
        let combatant_2_position = vector![1.0, 0.0, 3.0];
        // Combatant 3 is behind combatant 1
        let wall_position = vector![1.0, 0.0, 1.5];
        let wall_size = vector![5.0, 5.0, 0.5];

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
                Vector3::zero(),
                TeamAlignment::Home,
                rigid_body_set,
                collider_set,
            );

            let combatant_2 = CombatantObject::new(
                2,
                combatant_2_instance,
                combatant_2_position.clone(),
                Vector3::zero(),
                TeamAlignment::Home,
                rigid_body_set,
                collider_set,
            );

            let wall = ArenaBarrier::new(
                wall_position,
                wall_size,
                Default::default(),
                BarrierPathing::Disabled
            );
            let wall_rigid_body = wall.build_rigid_body().unwrap();
            let wall_collider = wall.build_collider().unwrap();
            let wall_rigid_body_handle = rigid_body_set.insert(wall_rigid_body);
            let wall_collider_handle = collider_set.insert_with_parent(
                wall_collider,
                wall_rigid_body_handle,
                rigid_body_set
            );

            // We must tick in order for the objects to be available in our tests
            physics_sim.tick();

            let mut active_colliders = CollidersMapT::new();
            let combatant_1_collider_handle = combatant_1.collider_handle.clone();
            active_colliders.insert(combatant_1.collider_handle, Combatant(combatant_1.id));
            active_colliders.insert(combatant_2.collider_handle, Combatant(combatant_2.id));
            active_colliders.insert(wall_collider_handle, Barrier);

            let mut combatants = CombatantsMapT::new();
            combatants.insert(combatant_1.id, combatant_1.clone());
            combatants.insert(combatant_2.id, combatant_2);

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
                1,
                combatant_1_collider_handle,
            );

            let combatant_forward_isometry = combatant_1.forward_isometry(rigid_body_set);

            let (_, new_beliefs) = field_of_view_sensor.sense(
                &combatant_forward_isometry,
                query_pipeline,
                rigid_body_set,
                collider_set,
                &active_colliders,
                &combatants,
                &BallsMapT::default(),
                1
            );

            let knows_combatant_2_position = new_beliefs.iter().any(|belief| {
                SatisfiableBelief::CombatantPosition()
                    .combatant_id(SatisfiableField::Exactly(2))
                    .satisfied_by(belief.belief.to_owned())
            });

            let no_direct_los = !new_beliefs.iter().any(|belief| {
                SatisfiableBelief::DirectLineOfSightToCombatant()
                    .satisfied_by(belief.belief.to_owned())
            });

            assert!(
                knows_combatant_2_position && no_direct_los,
                "knows_combatant_2_position={knows_combatant_2_position},\
                no_direct_los={no_direct_los}"
            );
        }
    }
}