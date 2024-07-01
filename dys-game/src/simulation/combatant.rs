use std::collections::HashMap;

use dys_world::arena::{navmesh::ArenaNavmesh, plate::PlateId};
use rapier3d::{dynamics::RigidBody, geometry::ColliderSet, math::Point, na::Matrix3x1, prelude::*};

use crate::{ai, game_objects::{ball::{BallId, BallObject}, combatant::{CombatantObject, CombatantState}, game_object::GameObject, plate::PlateObject}, game_state::GameState, game_tick::GameTickNumber};

use super::simulation_event::SimulationEvent;

pub(crate) fn simulate_combatants(game_state: &mut GameState) -> Vec<SimulationEvent> {
    let combatants = &mut game_state.combatants;

    let mut events = vec![];

    for (combatant_id, mut combatant_object) in combatants {
        let (rigid_body_set, collider_set) = game_state.physics_sim.sets_mut();

        ai::combatant_ai::process_combatant(combatant_object, &rigid_body_set, game_state.current_tick);

        let combatant_rb_handle = combatant_object.rigid_body_handle().expect("combatants should have a valid rigidbody handle");
        let combatant_rb = rigid_body_set.get_mut(combatant_rb_handle).expect("combatants rigid bodies should be registered with main set");
        
        events.extend(match combatant_object.combatant_state {
            CombatantState::Idle => simulate_state_idle(&mut combatant_object, game_state.current_tick),
            CombatantState::MovingToBall { ball_id } => simulate_moving_to_ball(&mut combatant_object, ball_id, &game_state.arena_navmesh, combatant_rb, &collider_set, &game_state.balls, game_state.current_tick),
            CombatantState::MovingToPlate { plate_id } => simulate_moving_to_plate(&mut combatant_object, plate_id, &game_state.arena_navmesh, combatant_rb, &collider_set, &game_state.plates, game_state.current_tick),
            CombatantState::RecoilingFromExplosion {} => simulate_state_recoiling_from_explosion(&mut combatant_object),
        });
    }

    events
}

fn simulate_state_idle(combatant_obj: &mut CombatantObject, current_tick: GameTickNumber) -> Vec<SimulationEvent> {
    // We should never be in the idle state
    vec![]
}

fn simulate_move(
    combatant_obj: &mut CombatantObject,
    arena_navmesh: &ArenaNavmesh,
    combatant_rb: &mut RigidBody,
    target_position: &Matrix3x1<f32>
) -> Point<f32> {
    let combatant_position = combatant_rb.translation();

    // ZJ-TODO: read this from combatant stats
    let mut total_distance_can_travel_this_tick = 2.0_f32;
    let mut new_combatant_position = combatant_position.to_owned();

    const UNIT_RESOLUTION: f32 = 1.0; // ZJ-TODO: read from config
    while total_distance_can_travel_this_tick >= UNIT_RESOLUTION {
        let Some(next_point) = arena_navmesh.get_next_point(new_combatant_position.into(), (*target_position).into()) else {
            break;
        };

        let lerp_distance = (total_distance_can_travel_this_tick - UNIT_RESOLUTION).clamp(0.0, UNIT_RESOLUTION);
        new_combatant_position = new_combatant_position.lerp(&next_point.coords, lerp_distance);
        total_distance_can_travel_this_tick = (total_distance_can_travel_this_tick - UNIT_RESOLUTION).max(0.0);
    }

    let should_wake_up = false;

    // ZJ-TODO: don't blindly copy original y
    //          this assumes we're on a perfectly flat plane
    new_combatant_position.y = combatant_position.y;
    combatant_rb.set_translation(new_combatant_position, should_wake_up);

    new_combatant_position.into()
}

fn simulate_moving_to_ball(
    combatant_obj: &mut CombatantObject,
    ball_id: BallId,
    arena_navmesh: &ArenaNavmesh,
    combatant_rb: &mut RigidBody,
    collider_set: &ColliderSet,
    balls: &HashMap<BallId, BallObject>,
    current_tick: GameTickNumber,
) -> Vec<SimulationEvent> {
    let Some(ball_object) = balls.get(&ball_id) else {
        panic!("failed to find BallObject to pathfind to");
    };

    let ball_collider_handle = ball_object.collider_handle().unwrap();
    let ball_collider = collider_set.get(ball_collider_handle).expect("failed to find ball collider");

    let ball_position = ball_collider.translation();
    let new_combatant_position = simulate_move(combatant_obj, arena_navmesh, combatant_rb, ball_position);

    // ZJ-TODO: if we're close enough to the ball, pick it up
    if are_points_equal(new_combatant_position, combatant_rb.translation().y, (*ball_position).into(), ball_position.y) {
        combatant_obj.change_state(current_tick, CombatantState::Idle);
    }

    vec![SimulationEvent::CombatantPositionUpdate {
        combatant_id: combatant_obj.id,
        position: new_combatant_position.coords,
    }]
}

fn simulate_moving_to_plate(
    combatant_obj: &mut CombatantObject,
    plate_id: PlateId,
    arena_navmesh: &ArenaNavmesh,
    combatant_rb: &mut RigidBody,
    collider_set: &ColliderSet,
    plates: &HashMap<PlateId, PlateObject>,
    current_tick: GameTickNumber,
) -> Vec<SimulationEvent>{
    let Some(plate_object) = plates.get(&plate_id) else {
        panic!("failed to find PlateObject to pathfind to");
    };

    let plate_collider_handle = plate_object.collider_handle().unwrap();
    let plate_collider = collider_set.get(plate_collider_handle).expect("failed to find plate collider");

    let plate_position = plate_collider.translation();
    let new_combatant_position = simulate_move(combatant_obj, arena_navmesh, combatant_rb, plate_position);

    if are_points_equal(new_combatant_position, combatant_rb.translation().y, (*plate_position).into(), plate_position.y) {
        combatant_obj.change_state(current_tick, CombatantState::Idle);
    }

    vec![SimulationEvent::CombatantPositionUpdate {
        combatant_id: combatant_obj.id,
        position: new_combatant_position.coords,
    }]
}

fn simulate_state_recoiling_from_explosion(_combatant_obj: &mut CombatantObject) -> Vec<SimulationEvent> {
    // A combatant's recovery from an explosion should be based on their stats
    vec![]
}

fn are_points_equal(a: Point<f32>, a_height: f32, b: Point<f32>, b_height: f32) -> bool {
    const CUSTOM_EPSILON: f32 = 0.03;
    let new_a = point![a.coords.x, a.coords.y - a_height, a.coords.z];
    let new_b = point![b.coords.x, b.coords.y - b_height, b.coords.z];

    (new_a - new_b).magnitude() <= CUSTOM_EPSILON
}