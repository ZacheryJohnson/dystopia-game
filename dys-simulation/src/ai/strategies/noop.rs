use std::sync::{Arc, Mutex};
use crate::{ai::{agent::Agent, belief::Belief, strategy::Strategy}, game_state::GameState, simulation::simulation_event::SimulationEvent};

pub(in crate::ai) struct NoopStrategy;

impl Strategy for NoopStrategy {
    fn name(&self) -> String {
        String::from("Noop")
    }

    fn can_perform(&self, _: &[Belief]) -> bool {
        true
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