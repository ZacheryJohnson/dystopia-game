use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::{ai::agent::Agent, game_state::GameState};
use crate::simulation::simulation_stage::SimulationStage;

pub(crate) fn simulate_combatants(
    game_state: Arc<Mutex<GameState>>
) -> SimulationStage {
    let start_time = Instant::now();

    let mut events = vec![];

    let combatants = {
        let game_state = game_state.lock().unwrap();
        game_state.combatants.clone()
    };

    // ZJ-TODO: do this in parallel
    for (_combatant_id, mut combatant_object) in combatants {
        events.append(&mut combatant_object.tick(game_state.clone()));
    }

    SimulationStage {
        pending_events: events,
        execution_duration: start_time.elapsed(),
    }
}
