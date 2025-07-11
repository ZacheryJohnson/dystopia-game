use std::sync::{Arc, Mutex};
use crate::{ai::{agent::Agent, strategy::Strategy}, game_state::GameState};
use crate::ai::beliefs::belief_set::BeliefSet;
use crate::simulation::simulation_event::PendingSimulationEvent;

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
    ) -> Option<Vec<PendingSimulationEvent>> {
        Some(vec![])
    }
}