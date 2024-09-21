use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

use crate::{ai::{agent::Agent, strategy::Strategy}, game_objects::combatant::CombatantState, game_state::GameState};

pub(in crate::ai) struct NoopStrategy;

impl Strategy for NoopStrategy {
    fn can_perform(&self) -> bool {
        true
    }

    fn is_complete(&self) -> bool {
        true
    }

    fn start(&mut self, agent: &mut dyn Agent, _game_state: &mut GameState) {
        // no-op
    }

    fn tick(
        &mut self,
        agent: &mut dyn Agent,
        game_state: &mut GameState,
    ) {
        // no-op
    }

    fn stop(&mut self, agent: &mut dyn Agent, _game_state: &mut GameState) {
        // no-op
    }
}