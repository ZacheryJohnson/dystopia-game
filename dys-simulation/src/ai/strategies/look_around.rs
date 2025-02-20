use std::f32::consts::FRAC_PI_2;
use std::sync::{Arc, Mutex};
use rapier3d::na::{Rotation3, Vector3};
use crate::{ai::{agent::Agent, strategy::Strategy}, game_state::GameState, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::BeliefSet;

pub struct LookAroundStrategy {
    is_complete: bool,
}

impl LookAroundStrategy {
    pub fn new() -> LookAroundStrategy {
        LookAroundStrategy {
            is_complete: false,
        }
    }
}

impl Strategy for LookAroundStrategy {
    fn name(&self) -> String {
        String::from("Look Around")
    }

    fn can_perform(&self, _: &BeliefSet) -> bool {
        true
    }

    fn should_interrupt(&self, _: &BeliefSet) -> bool {
        false
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    #[tracing::instrument(
        name = "look_around::tick",
        fields(combatant_id = agent.combatant().id),
        skip_all,
        level = "trace"
    )]
    fn tick(
        &mut self,
        agent: &dyn Agent,
        game_state: Arc<Mutex<GameState>>,
    ) -> Option<Vec<SimulationEvent>> {
        let events = vec![];

        let mut game_state = game_state.lock().unwrap();
        let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();

        let combatant_rb = rigid_body_set.get_mut(agent.combatant().rigid_body_handle).unwrap();

        let current_rotation = combatant_rb.rotation().to_owned();
        let new_rotation = current_rotation * Rotation3::from_axis_angle(
            &Vector3::y_axis(),
            FRAC_PI_2,
        );

        combatant_rb.set_rotation(
            new_rotation,
            true
        );
        self.is_complete = true;

        Some(events)
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::StdRng;
    use rand::SeedableRng;
    use rand_distr::num_traits::Zero;
    use rapier3d::prelude::*;
    use rapier3d::na::{vector, UnitQuaternion, Vector3};
    use crate::ai::agent::Agent;
    use crate::ai::strategies::look_around::LookAroundStrategy;
    use crate::ai::strategy::Strategy;
    use crate::ai::test_utils::{make_test_game_state, TestAgent};
    use crate::game_objects::combatant::{CombatantObject, TeamAlignment};
    use crate::game_objects::game_object_type::GameObjectType::Combatant;
    use crate::game_state::{CollidersMapT, CombatantsMapT};
    use crate::generator::Generator;
    use crate::physics_sim::PhysicsSim;

    #[test]
    fn rotates_90_degrees_every_tick() {
        let world = Generator::new().generate_world(&mut StdRng::from_entropy());
        let combatant_1_instance = world.combatants[0].clone();

        let mut physics_sim = PhysicsSim::new(10);
        let combatant_1 = {
            let (
                rigid_body_set,
                collider_set,
                _,
            ) = physics_sim.sets_mut();

            let combatant_1 = CombatantObject::new(
                1,
                combatant_1_instance,
                vector![0.0, 0.0, 0.0],
                Vector3::zero(),
                TeamAlignment::Home,
                rigid_body_set,
                collider_set,
            );
            // We must tick in order for the objects to be available in our tests
            physics_sim.tick();

            let mut active_colliders = CollidersMapT::new();
            active_colliders.insert(combatant_1.collider_handle, Combatant(combatant_1.id));

            let mut combatants = CombatantsMapT::new();
            combatants.insert(combatant_1.id, combatant_1.clone());

            combatant_1
        };

        let mut strategy = LookAroundStrategy::new();
        let test_agent = TestAgent::from_combatant(combatant_1);
        let test_game_state = make_test_game_state(Some(physics_sim));

        {
            let game_state = test_game_state.lock().unwrap();
            let (rigid_body_set, _, _) = game_state.physics_sim.sets();
            let combatant_rb = rigid_body_set.get(test_agent.combatant().rigid_body_handle).unwrap();
            assert_eq!(
                combatant_rb.rotation(),
                &UnitQuaternion::<f32>::from_axis_angle(&Vector3::y_axis(), 0.0)
            );
        }

        // Do 4 rotations and ensure we're looking at increments of 90 degrees
        for angle in vec![90_f32.to_radians(), 180_f32.to_radians(), 270_f32.to_radians(), 0_f32.to_radians()] {
            let result = strategy.tick(&test_agent, test_game_state.clone());
            assert!(result.is_some());
            assert_eq!(result.unwrap().len(), 0);

            let mut game_state = test_game_state.lock().unwrap();
            game_state.physics_sim.tick();
            let (rigid_body_set, _, _) = game_state.physics_sim.sets();
            let combatant_rb = rigid_body_set.get(test_agent.combatant().rigid_body_handle).unwrap();
            let expected_rotation = UnitQuaternion::<f32>::from_axis_angle(&Vector3::y_axis(), angle);

            let angle_diff = expected_rotation.angle() - combatant_rb.rotation().angle();
            assert!(angle_diff <= f32::EPSILON);
        }
    }
}