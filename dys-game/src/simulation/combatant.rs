use std::collections::HashMap;

use dys_world::arena::{navmesh::ArenaNavmesh, plate::PlateId};
use rapier3d::{dynamics::{RigidBody, RigidBodySet}, geometry::{ColliderHandle, ColliderSet}};

use crate::{game_objects::{ball::BallId, combatant::{CombatantObject, CombatantState}, game_object::GameObject, game_object_type::GameObjectType, plate::PlateObject}, game_state::GameState, game_tick::GameTickNumber};

use super::simulation_event::SimulationEvent;

pub(crate) fn simulate_combatants(game_state: &mut GameState) -> Vec<SimulationEvent> {
    let combatants = &mut game_state.combatants;

    let mut events = vec![];

    for (combatant_id, mut combatant_object) in combatants {
        let combatant_rb_handle = combatant_object.rigid_body_handle().expect("combatants should have a valid rigidbody handle");

        let (rigid_body_set, collider_set) = game_state.physics_sim.sets();

        let combatant_rb = rigid_body_set.get(combatant_rb_handle).expect("combatants rigid bodies should be registered with main set");
        
        events.extend(match combatant_object.combatant_state {
            CombatantState::Idle => simulate_state_idle(&mut combatant_object, game_state.current_tick),
            CombatantState::MovingToBall { ball_id } => simulate_moving_to_ball(&mut combatant_object, ball_id),
            CombatantState::MovingToPlate { plate_id } => simulate_moving_to_plate(&mut combatant_object, plate_id, &game_state.arena_navmesh, &combatant_rb, &collider_set, &game_state.plates),
            CombatantState::RecoilingFromExplosion {} => simulate_state_recoiling_from_explosion(&mut combatant_object),
        });

        if combatant_object.is_dirty() {
            events.push(SimulationEvent::CombatantPositionUpdate { combatant_id: *combatant_id, position: *combatant_rb.translation() });
        }
    }

    events
}

fn simulate_state_idle(combatant_obj: &mut CombatantObject, current_tick: GameTickNumber) -> Vec<SimulationEvent> {
    // We should never be idle for more than one tick. Figure out something productive to do

    // ZJ-TODO: delete the following, temp to test scoring + pathfinding
    combatant_obj.change_state(current_tick, CombatantState::MovingToPlate { plate_id: 1 });

    vec![]
}

fn simulate_moving_to_ball(combatant_obj: &mut CombatantObject, ball_id: BallId) -> Vec<SimulationEvent> {
    vec![]
}

fn simulate_moving_to_plate(
    combatant_obj: &mut CombatantObject,
    plate_id: PlateId,
    arena_navmesh: &ArenaNavmesh,
    combatant_rb: &RigidBody,
    collider_set: &ColliderSet,
    plates: &HashMap<PlateId, PlateObject>
) -> Vec<SimulationEvent>{
    // ZJ-TODO: pathfind to plate

    /*
    let Some(plate_object) = plates.get(&plate_id) else {
        panic!("failed to find plate object to pathfind to");
    };

    let plate_collider_handle = plate_object.collider_handle().unwrap();
    let plate_collider = collider_set.get(plate_collider_handle).expect("failed to find plate collider");

    let plate_position = plate_collider.translation();
    let combatant_position = combatant_rb.translation();    

    // ZJ-TODO: we shouldn't calculate the path every time in this function
    //          we should calculate once, cache it, and continue traversing that path until we're at the goal
    let path = arena_navmesh.create_path((*combatant_position).into(), (*plate_position).into());
    */

    vec![]
}

fn simulate_state_recoiling_from_explosion(combatant_obj: &mut CombatantObject) -> Vec<SimulationEvent> {
    // A combatant's recovery from an explosion should be based on their stats
    vec![]
}