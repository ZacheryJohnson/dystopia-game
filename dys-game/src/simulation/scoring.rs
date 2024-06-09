use rapier3d::{geometry::ColliderHandle, pipeline::QueryFilter};

use crate::{game_objects::{combatant::TeamAlignment, game_object_type::GameObjectType}, game_state::GameState};

use super::simulation_event::SimulationEvent;

const PLATE_POINTS_PER_TICK: u8 = 1; // ZJ-TODO: move this to config
const OWNED_PLATE_MULTIPLIER: u8 = 2; // ZJ-TODO: move this to config

pub fn simulate_scoring(
    game_state: &mut GameState,
) -> Vec<SimulationEvent> {
    let (query_pipeline, rigid_body_set, collider_set) = game_state.physics_sim.query_pipeline_and_sets();

    let plate_object_colliders: Vec<(&ColliderHandle, &GameObjectType)> = game_state
        .active_colliders
        .iter()
        .filter(|(_, game_object_type)| matches!(game_object_type, &GameObjectType::Plate(_)))
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
        let mut scoring_combatants = vec![];
        for collider_handle in affected_colliders {
            let GameObjectType::Combatant(combatant_id) = game_state.active_colliders.get(&collider_handle).unwrap() else {
                continue;
            };

            let Some(combatant) = game_state.combatants.get(combatant_id) else {
                continue;
            };

            scoring_combatants.push((combatant_id, combatant.team));
        }

        let home_owns_plate = scoring_combatants.iter().all(|(_, team)| *team == TeamAlignment::Home);
        let away_owns_plate = scoring_combatants.iter().all(|(_, team)| *team == TeamAlignment::Away);

        let one_team_owns_plate = home_owns_plate || away_owns_plate;

        for (combatant_id, _) in scoring_combatants {
            simulation_events.push(SimulationEvent::PointsScoredByCombatant { 
                plate_id: *plate_id, 
                combatant_id: *combatant_id,
                points: if one_team_owns_plate { PLATE_POINTS_PER_TICK * OWNED_PLATE_MULTIPLIER } else { PLATE_POINTS_PER_TICK },
            });
        }
    }

    simulation_events
}