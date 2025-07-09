use std::sync::{Arc, Mutex};
use std::time::Instant;
use rapier3d::{geometry::ColliderHandle, pipeline::QueryFilter};

use crate::{game_objects::{combatant::TeamAlignment, game_object_type::GameObjectType}, game_state::GameState};
use crate::simulation::simulation_stage::SimulationStage;
use super::simulation_event::{PendingSimulationEvent, SimulationEvent};

const PLATE_POINTS_PER_TICK: u8 = 1; // ZJ-TODO: move this to config
const OWNED_PLATE_MULTIPLIER: u8 = 2; // ZJ-TODO: move this to config

pub fn simulate_scoring(
    game_state: Arc<Mutex<GameState>>,
) -> SimulationStage {
    let start_time = Instant::now();

    let active_colliders = {
        let game_state = game_state.lock().unwrap();
        game_state.active_colliders.clone()
    };

    let plate_object_colliders: Vec<(&ColliderHandle, &GameObjectType)> = active_colliders
        .iter()
        .filter(|(_, game_object_type)| matches!(game_object_type, &GameObjectType::Plate(_)))
        .collect();

    let mut simulation_events = vec![];
    for (collider_handle, game_object_type) in plate_object_colliders {
        let GameObjectType::Plate(plate_id) = game_object_type else {
            panic!("non-plate game object returned as game object type");
        };

        let mut affected_colliders = vec![];
        {
            let mut game_state = game_state.lock().unwrap();
            let (query_pipeline, rigid_body_set, collider_set) = game_state.physics_sim.query_pipeline_and_sets();
            let plate_collider = collider_set.get(*collider_handle).expect("failed to find plate with collider handle");
            let plate_shape = plate_collider.shape();
            let plate_isometry = plate_collider.position();
            let query_filter = QueryFilter::only_dynamic()
                .exclude_sensors();
            // ZJ-TODO: use InteractionGroups to get only combatants and ignore everything else

            query_pipeline.intersections_with_shape(rigid_body_set, collider_set, plate_isometry, plate_shape, query_filter, |handle| {
                affected_colliders.push(handle);
                true // return true to continue iterating over collisions
            });
        }

        let (active_colliders, combatants) = {
            let game_state = game_state.lock().unwrap();

            let active_colliders = game_state.active_colliders.clone();
            let combatants = game_state.combatants.clone();

            (active_colliders, combatants)
        };

        // If plate score < 0, away team controls the plate; if > 0, home team controls the plate
        let mut scoring_combatants = vec![];
        for collider_handle in affected_colliders {
            let GameObjectType::Combatant(combatant_id) = active_colliders.get(&collider_handle).unwrap() else {
                continue;
            };

            let Some(combatant) = combatants.get(combatant_id) else {
                continue;
            };

            scoring_combatants.push((combatant_id, combatant.team));
        }

        let home_owns_plate = scoring_combatants.iter().all(|(_, team)| *team == TeamAlignment::Home);
        let away_owns_plate = scoring_combatants.iter().all(|(_, team)| *team == TeamAlignment::Away);

        let one_team_owns_plate = home_owns_plate || away_owns_plate;

        for (combatant_id, _) in scoring_combatants {
            simulation_events.push(PendingSimulationEvent(
                SimulationEvent::PointsScoredByCombatant {
                    plate_id: *plate_id,
                    combatant_id: *combatant_id,
                    points: if one_team_owns_plate { PLATE_POINTS_PER_TICK * OWNED_PLATE_MULTIPLIER } else { PLATE_POINTS_PER_TICK },
                }
            ));
        }
    }

    SimulationStage {
        execution_duration: start_time.elapsed(),
        pending_events: simulation_events
    }
}