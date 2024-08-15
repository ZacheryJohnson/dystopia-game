use crate::{ai::goap::strategy::Strategy, game_objects::combatant::CombatantObject, game_state::GameState};

pub(in crate::ai::goap) struct NoopStrategy;

impl Strategy for NoopStrategy {
    fn can_perform(&self) -> bool {
        true
    }

    fn is_complete(&self) -> bool {
        true
    }

    fn start(&mut self, _combatant: &mut CombatantObject, _game_state: &mut GameState) {
        // no-op
    }

    fn tick(&mut self, _combatant: &mut CombatantObject, _game_state: &mut GameState) {
        // no-op
    }

    fn stop(&mut self, _combatant: &mut CombatantObject, _game_state: &mut GameState) {
        // no-op
    }
}