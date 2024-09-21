use crate::{ai::agent::Agent, game_state::{CombatantsMapT, GameState}};

use super::simulation_event::SimulationEvent;

pub(crate) fn simulate_combatants(combatants: &mut CombatantsMapT, game_state: &mut GameState) -> Vec<SimulationEvent> {
    let mut events = vec![];

    for (_combatant_id, combatant_object) in combatants {
        events.append(&mut combatant_object.tick(game_state));
    }

    events
}
