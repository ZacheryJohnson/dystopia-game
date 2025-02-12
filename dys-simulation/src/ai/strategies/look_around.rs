use std::sync::{Arc, Mutex};
use rapier3d::math::Rotation;
use rapier3d::prelude::*;
use rapier3d::na::vector;
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

        combatant_rb.set_rotation(
            Rotation::from_scaled_axis(vector![0.0, 180.0_f32.to_radians(), 0.0]),
            true
        );
        self.is_complete = true;

        Some(events)
    }
}