use std::sync::{Arc, Mutex};
use std::time::Instant;
use dys_satisfiable::SatisfiableField;
use crate::ai::agent::Agent;
use crate::ai::belief::SatisfiableBelief;
use crate::game_state::GameState;
use crate::simulation::simulation_event::SimulationEvent;
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
            let mut game_state = game_state.lock().unwrap();
            let active_colliders = game_state.active_colliders.clone();
            let combatants = game_state.combatants.clone();
            let balls = game_state.balls.clone();
            let (
                query_pipeline,
                rigid_body_set,
                collider_set
            ) = game_state.physics_sim.query_pipeline_and_sets();

            let combatant_isometry = combatant_object.forward_isometry(rigid_body_set);

            let sensors = {
                let mut combatant_state = combatant_object.combatant_state.lock().unwrap();
                // ZJ-TODO: refactor yuck
                if combatant_state.stunned_by_explosion {
                    vec![]
                } else {
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
            for (sensor_id, sensor) in sensors {
                let new_beliefs = sensor.sense(
                    &combatant_isometry,
                    query_pipeline,
                    rigid_body_set,
                    collider_set,
                    &active_colliders,
                    &combatants,
                    &balls,
                    current_tick);

                let mut combatant_state = combatant_object.combatant_state.lock().unwrap();
                combatant_state.beliefs.add_expiring_beliefs_from_source(
                    sensor_id,
                    &new_beliefs,
                );
            }
        }

        let mut combatant_events = combatant_object.tick(game_state.clone());

        let maybe_position_update = combatant_events
            .iter()
            .find(|evt| matches!(evt, SimulationEvent::CombatantPositionUpdate {..}));

        if maybe_position_update.is_none() {
            let game_state = game_state.lock().unwrap();
            let (rigid_body_set, _, _) = game_state.physics_sim.sets();
            let combatant_translation = rigid_body_set
                .get(combatant_object.rigid_body_handle)
                .unwrap()
                .translation();

            events.push(SimulationEvent::CombatantPositionUpdate {
                combatant_id: *combatant_id,
                position: *combatant_translation,
            });
        }

        events.append(&mut combatant_events);
    }

    SimulationStage {
        pending_events: events,
        execution_duration: start_time.elapsed(),
    }
}
