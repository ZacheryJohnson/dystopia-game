use crate::game_state::GameState;

use super::agent::Agent;

pub trait Strategy {
    /// Can this strategy be performed given the current state of the world?
    fn can_perform(&self) -> bool;

    /// Was this strategy running and is now complete?
    fn is_complete(&self) -> bool;

    fn start(&mut self, agent: &mut dyn Agent, game_state: &mut GameState);

    fn tick(&mut self, agent: &mut dyn Agent, game_state: &mut GameState);

    fn stop(&mut self, agent: &mut dyn Agent, game_state: &mut GameState);
}