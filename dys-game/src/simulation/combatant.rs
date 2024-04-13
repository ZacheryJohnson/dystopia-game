use crate::{game_objects::game_object::GameObject, game_state::GameState};

use super::simulation_event::SimulationEvent;

pub(crate) fn simulate_combatants(game_state: &mut GameState) -> Vec<SimulationEvent> {
    let combatants = &game_state.combatants;

    let mut events = vec![];

    for (combatant_id, combatant_object) in combatants {
        let combatant_rb_handle = combatant_object.rigid_body_handle().expect("combatants should have a valid rigidbody handle");

        let (rigid_body_set, _) = game_state.physics_sim.sets();

        let combatant_rb = rigid_body_set.get(combatant_rb_handle).expect("combatants rigid bodies should be registered with main set");
        events.push(SimulationEvent::CombatantPositionUpdate { combatant_id: *combatant_id, position: *combatant_rb.translation() });
    }

    events
}