use std::collections::HashMap;

use dys_world::arena::{navmesh::ArenaNavmesh, plate::PlateId};
use rapier3d::{dynamics::RigidBody, geometry::ColliderSet, math::Point, na::Matrix3x1, prelude::*};

use crate::{ai::{self, agent::Agent}, game_objects::{ball::{BallId, BallObject}, combatant::{CombatantObject, CombatantState}, game_object::GameObject, plate::PlateObject}, game_state::{CombatantsMapT, GameState}, game_tick::GameTickNumber};

use super::simulation_event::SimulationEvent;

pub(crate) fn simulate_combatants(combatants: &mut CombatantsMapT, game_state: &mut GameState) -> Vec<SimulationEvent> {
    let mut events = vec![];

    for (_combatant_id, mut combatant_object) in combatants {
        // let (rigid_body_set, collider_set, joint_set) = game_state.physics_sim.sets_mut();

        // ai::combatant_ai::process_combatant(combatant_object, &rigid_body_set, &collider_set, &mut game_state.plates, &mut game_state.balls, game_state.current_tick);
        
        // events.extend(match combatant_object.combatant_state {
        //     CombatantState::Idle => simulate_state_idle(&mut combatant_object, game_state.current_tick),
        //     CombatantState::MovingToBall { ball_id } => simulate_moving_to_ball(&mut combatant_object, ball_id, &game_state.arena_navmesh, rigid_body_set, joint_set, &mut game_state.balls, game_state.current_tick),
        //     CombatantState::MovingToPlate { plate_id } => simulate_moving_to_plate(&mut combatant_object, plate_id, &game_state.arena_navmesh, rigid_body_set, &collider_set, &game_state.plates, game_state.current_tick),
        //     CombatantState::RecoilingFromExplosion {} => simulate_state_recoiling_from_explosion(&mut combatant_object, &rigid_body_set, game_state.current_tick),
        // });

        combatant_object.tick(game_state);
    }

    events
}

fn simulate_state_idle(_combatant_obj: &mut CombatantObject, _current_tick: GameTickNumber) -> Vec<SimulationEvent> {
    // We should never be in the idle state
    vec![]
}

fn simulate_move(
    _combatant_obj: &mut CombatantObject,
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

    // ZJ-TODO: don't blindly copy original y
    //          this assumes we're on a perfectly flat plane
    new_combatant_position.y = combatant_position.y;
    combatant_rb.set_translation(new_combatant_position, true);
    //combatant_rb.set_next_kinematic_translation(new_combatant_position);

    new_combatant_position.into()
}

fn simulate_moving_to_ball(
    combatant_obj: &mut CombatantObject,
    ball_id: BallId,
    arena_navmesh: &ArenaNavmesh,
    rigid_body_set: &mut RigidBodySet,
    joint_set: &mut MultibodyJointSet,
    balls: &mut HashMap<BallId, BallObject>,
    current_tick: GameTickNumber,
) -> Vec<SimulationEvent> {
    let Some(ball_object) = balls.get_mut(&ball_id) else {
        panic!("failed to find BallObject to pathfind to");
    };

    // If someone else is holding the ball, change our state back to idle
    if ball_object.held_by.is_some() {
        combatant_obj.change_state(current_tick, CombatantState::Idle);
        return vec![];
    }

    let ball_rigid_body_handle = ball_object.rigid_body_handle().unwrap();
    let ball_position = {
        let ball_rb = rigid_body_set.get(ball_rigid_body_handle).unwrap();
        ball_rb.translation().to_owned()
    };

    let combatant_rb = rigid_body_set.get_mut(combatant_obj.rigid_body_handle().unwrap()).unwrap();
    let new_combatant_position = simulate_move(combatant_obj, arena_navmesh, combatant_rb, &ball_position);

    if are_points_equal(new_combatant_position, combatant_rb.translation().y, ball_position.into(), ball_position.y) {
        combatant_obj.pickup_ball(ball_id);
        ball_object.set_held_by(Some(combatant_obj.id), current_tick);
        
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
    rigid_body_set: &mut RigidBodySet,
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
    
    let combatant_rb = rigid_body_set.get_mut(combatant_obj.rigid_body_handle().unwrap()).unwrap();
    let new_combatant_position = simulate_move(combatant_obj, arena_navmesh, combatant_rb, plate_position);

    if are_points_equal(new_combatant_position, combatant_rb.translation().y, (*plate_position).into(), plate_position.y) {
        combatant_obj.change_state(current_tick, CombatantState::Idle);
    }

    vec![SimulationEvent::CombatantPositionUpdate {
        combatant_id: combatant_obj.id,
        position: new_combatant_position.coords,
    }]
}

fn simulate_state_recoiling_from_explosion(
    combatant_obj: &mut CombatantObject,
    rigid_body_set: &RigidBodySet,
    current_tick: GameTickNumber,
) -> Vec<SimulationEvent> {
    let combatant_rb = rigid_body_set.get(combatant_obj.rigid_body_handle().unwrap()).unwrap();
    let combatant_position = combatant_rb.translation();

    try_recover_from_explosion(combatant_obj, &combatant_rb, current_tick);

    vec![SimulationEvent::CombatantPositionUpdate {
        combatant_id: combatant_obj.id,
        position: *combatant_position,
    }]
}

fn are_points_equal(a: Point<f32>, a_height: f32, b: Point<f32>, b_height: f32) -> bool {
    const CUSTOM_EPSILON: f32 = 0.03;
    let new_a = point![a.coords.x, a.coords.y - a_height, a.coords.z];
    let new_b = point![b.coords.x, b.coords.y - b_height, b.coords.z];

    (new_a - new_b).magnitude() <= CUSTOM_EPSILON
}

fn try_recover_from_explosion(combatant_obj: &mut CombatantObject, combatant_rb: &RigidBody, current_tick: GameTickNumber) {
    match combatant_obj.combatant_state {
        CombatantState::RecoilingFromExplosion {  } => {},
        _ => return
    };

    const KINETIC_ENERGY_THRESHOLD: f32 = 3.0;
    if combatant_rb.kinetic_energy() >= KINETIC_ENERGY_THRESHOLD {
        return;
    }

    // Combatant is too slow - consider them recovered
    combatant_obj.change_state(current_tick, CombatantState::Idle);
}