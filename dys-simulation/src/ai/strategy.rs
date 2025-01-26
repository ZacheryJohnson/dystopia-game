use std::sync::{Arc, Mutex};
use crate::{game_state::GameState, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::BeliefSet;
use super::agent::Agent;

pub trait Strategy {
    fn name(&self) -> String;

    /// Can this strategy be performed given our current beliefs about the world?
    fn can_perform(&self, owned_beliefs: &BeliefSet) -> bool;

    /// Should we interrupt the entire plan, forcing a re-plan?
    fn should_interrupt(&self, owned_beliefs: &BeliefSet) -> bool;

    /// Was this strategy running and is now complete?
    fn is_complete(&self) -> bool;

    /// Run the strategy on the agent given the game state,
    /// returning a collection of pending events that happened during the tick.
    /// The returned events have not yet been executed.
    /// If the strategy fails to execute successfully, None will be returned.
    /// Otherwise, failure has not been encountered, and any simulation events are returned.
    fn tick(
        &mut self,
        agent: &dyn Agent,
        game_state: Arc<Mutex<GameState>>
    ) -> Option<Vec<SimulationEvent>>;
}