use crate::{game_state::GameState, simulation::simulation_event::SimulationEvent};

use super::{agent::Agent, belief::Belief};

pub trait Strategy {
    fn name(&self) -> String;

    /// Can this strategy be performed given our current beliefs about the world?
    fn can_perform(&self, owned_beliefs: &[Belief]) -> bool;

    /// Was this strategy running and is now complete?
    fn is_complete(&self) -> bool;

    /// Run the strategy on the agent given the game state,
    /// returning a collection of events that happened during the tick.
    fn tick(&mut self, agent: &mut dyn Agent, game_state: &mut GameState) -> Vec<SimulationEvent>;
}