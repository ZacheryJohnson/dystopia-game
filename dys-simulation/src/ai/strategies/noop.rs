use std::sync::{Arc, Mutex};
use crate::{ai::{agent::Agent, strategy::Strategy}, game_state::GameState, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::BeliefSet;

pub(in crate::ai) struct NoopStrategy;

impl Strategy for NoopStrategy {
    fn name(&self) -> String {
        String::from("Noop")
    }

    fn can_perform(&self, _: &BeliefSet) -> bool {
        true
    }

    fn should_interrupt(&self, _: &BeliefSet) -> bool {
        false
    }

    fn is_complete(&self) -> bool {
        true
    }

    fn tick(
        &mut self,
        _: &dyn Agent,
        _: Arc<Mutex<GameState>>,
    ) -> Option<Vec<SimulationEvent>> {
        Some(vec![])
    }
}