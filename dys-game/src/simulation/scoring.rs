use rapier3d::{geometry::ColliderHandle, pipeline::QueryFilter};

use crate::{game_objects::{combatant::TeamAlignment, game_object_type::GameObjectType}, game_state::GameState};

use super::simulation_event::SimulationEvent;

pub fn simulate_scoring(
    game_state: &mut GameState,
) -> Vec<SimulationEvent> {
    let (query_pipeline, rigid_body_set, collider_set) = game_state.physics_sim.query_pipeline_and_sets();

    let plate_object_colliders: Vec<(&ColliderHandle, &GameObjectType)> = game_state
        .active_colliders
        .iter()
        .filter(|kv| matches!(kv.1, &GameObjectType::Plate(_)))
        .collect();

    let mut simulation_events = vec![];
    for (collider_handle, game_object_type) in plate_object_colliders {
        let GameObjectType::Plate(plate_id) = game_object_type else {
            panic!("non-plate game object returned as game object type");
        };

        let plate_collider = collider_set.get(*collider_handle).expect("failed to find plate with collider handle");
        let plate_shape = plate_collider.shape();
        let plate_isometry = plate_collider.position();
        let query_filter = QueryFilter::only_dynamic()
            .exclude_sensors();
        // ZJ-TODO: use InteractionGroups to get only combatants and ignore everything else

        let mut affected_colliders = vec![];
        query_pipeline.intersections_with_shape(rigid_body_set, collider_set, plate_isometry, plate_shape, query_filter, |handle| {
            affected_colliders.push(handle);
            true // return true to continue iterating over collisions
        });

        // If plate score < 0, away team controls the plate; if > 0, home team controls the plate
        let mut plate_score: i8 = 0;
        for collider_handle in affected_colliders {
            let GameObjectType::Combatant(combatant_id) = game_state.active_colliders.get(&collider_handle).unwrap() else {
                continue;
            };

            let Some(combatant) = game_state.combatants.get(combatant_id) else {
                continue;
            };

            plate_score += if combatant.team == TeamAlignment::Away { -1 } else { 1 };
        }

        // No points are scored if both teams have an equivalent number of combatants on the plate (or no combatants on the plate)
        if plate_score == 0 {
            continue;
        }

        simulation_events.push(SimulationEvent::PointsScoredOnPlate { 
            plate_id: *plate_id, 
            points: plate_score.abs() as u8, 
            team: if plate_score < 0 { TeamAlignment::Away } else { TeamAlignment::Home }
        });
    }

    simulation_events
}