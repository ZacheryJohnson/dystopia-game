use std::sync::{Arc, Mutex};
use crate::{ai::agent::Agent, game_state::GameState};

use super::simulation_event::SimulationEvent;

pub(crate) fn simulate_combatants(
    game_state: Arc<Mutex<GameState>>
) -> Vec<SimulationEvent> {
    let mut events = vec![];

    let combatants = {
        let game_state = game_state.lock().unwrap();
        game_state.combatants.clone()
    };

    // ZJ-TODO: do this in parallel
    for (_combatant_id, mut combatant_object) in combatants {
        events.append(&mut combatant_object.tick(game_state.clone()));
    }

    events
}
