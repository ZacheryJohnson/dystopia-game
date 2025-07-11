use std::sync::{Arc, Mutex};
use std::time::Instant;
use dys_satisfiable::SatisfiableField;
use crate::ai::agent::Agent;
use crate::ai::belief::SatisfiableBelief;
use crate::game_state::GameState;
use crate::simulation::simulation_event::{PendingSimulationEvent, SimulationEvent};
use crate::simulation::simulation_stage::SimulationStage;

pub(crate) fn simulate_combatants(
    game_state: Arc<Mutex<GameState>>
) -> SimulationStage {
    let start_time = Instant::now();

    let mut events = vec![];

    let current_tick = game_state.lock().unwrap().current_tick.to_owned();
    let mut combatants = {
        let game_state = game_state.lock().unwrap();
        game_state.combatants.clone()
    };

    // Update combatants' sensors
    for (combatant_id, combatant_object) in &mut combatants {
        {
            let sensors = {
                // ZJ-TODO: refactor yuck
                if combatant_object.is_stunned() {
                    vec![]
                } else {
                    let mut combatant_state = combatant_object.combatant_state.lock().unwrap();
                    combatant_state.beliefs.expire_stale_beliefs(current_tick);

                    // ZJ-TODO: don't like this, have the simulation be correct elsewhere
                    if combatant_state.holding_ball.is_none() {
                        combatant_state.beliefs.remove_beliefs_by_test(
                            &SatisfiableBelief::HeldBall()
                                .combatant_id(SatisfiableField::Exactly(combatant_id.to_owned()))
                        );
                    }

                    combatant_state.sensors.iter()
                        .map(|(id, sensor)| (id.to_owned(), sensor.to_owned()))
                        .collect::<Vec<_>>()
                }

            };

            let combatant_isometry = {
                let game_state = game_state.lock().unwrap();
                let (rigid_body_set, _, _) = game_state.physics_sim.sets();
                combatant_object.forward_isometry(rigid_body_set)
            };

            for (sensor_id, sensor) in sensors {
                let (should_interrupt, new_beliefs) = sensor.sense(
                    &combatant_isometry,
                    game_state.clone()
                );

                let mut combatant_state = combatant_object.combatant_state.lock().unwrap();
                combatant_state.beliefs.add_expiring_beliefs_from_source(
                    sensor_id,
                    &new_beliefs,
                );

                if should_interrupt {
                    combatant_state.plan.clear();
                    combatant_state.current_action = None;
                }
            }
        }

        let mut combatant_events = combatant_object.tick(game_state.clone());

        let maybe_position_update = combatant_events
            .iter()
            .find(|evt| matches!(evt, PendingSimulationEvent(SimulationEvent::CombatantPositionUpdate {..})));

        if maybe_position_update.is_none() {
            let game_state = game_state.lock().unwrap();
            let (rigid_body_set, _, _) = game_state.physics_sim.sets();
            let combatant_translation = rigid_body_set
                .get(combatant_object.rigid_body_handle)
                .unwrap()
                .translation();

            events.push(PendingSimulationEvent(
                SimulationEvent::CombatantPositionUpdate {
                    combatant_id: *combatant_id,
                    position: *combatant_translation,
                }
            ));
        }

        events.append(&mut combatant_events);
    }

    SimulationStage {
        pending_events: events,
        execution_duration: start_time.elapsed(),
    }
}
