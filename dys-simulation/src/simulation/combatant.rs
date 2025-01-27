use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::{ai::{agent::Agent, sensor::Sensor}, game_state::GameState};
use crate::simulation::simulation_stage::SimulationStage;

pub(crate) fn simulate_combatants(
    game_state: Arc<Mutex<GameState>>
) -> SimulationStage {
    let start_time = Instant::now();

    let mut events = vec![];

    let mut combatants = {
        let game_state = game_state.lock().unwrap();
        game_state.combatants.clone()
    };

    // Update combatants' sensors
    for (_, combatant_object) in &mut combatants {
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

            for (sensor_id, sensor) in combatant_object.sensors() {
                let new_beliefs = sensor.sense(
                    &combatant_isometry,
                    query_pipeline,
                    rigid_body_set,
                    collider_set,
                    &active_colliders,
                    &combatants,
                    &balls);

                let mut combatant_state = combatant_object.combatant_state.lock().unwrap();
                combatant_state.beliefs.remove_beliefs_from_source(sensor_id);
                combatant_state.beliefs.add_beliefs_from_source(sensor_id, &new_beliefs);
            }
        }

        events.append(&mut combatant_object.tick(game_state.clone()));
    }

    SimulationStage {
        pending_events: events,
        execution_duration: start_time.elapsed(),
    }
}
