use crate::{game_objects::combatant::CombatantObject, game_state::GameState};

pub(super) trait Strategy {
    /// Can this strategy be performed given the current state of the world?
    fn can_perform(&self) -> bool;

    /// Was this strategy running and is now complete?
    fn is_complete(&self) -> bool;

    fn start(&mut self, combatant: &mut CombatantObject, game_state: &mut GameState);

    fn tick(&mut self, combatant: &mut CombatantObject, game_state: &mut GameState);

    fn stop(&mut self, combatant: &mut CombatantObject, game_state: &mut GameState);
}