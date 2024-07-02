use dys_world::arena::plate::PlateId;
use rapier3d::dynamics::RigidBodySet;

use crate::{game_objects::{ball::BallId, combatant::{CombatantObject, CombatantState}, game_object::GameObject}, game_tick::GameTickNumber};

pub fn process_combatant(
    combatant_object: &mut CombatantObject,
    rigid_body_set: &RigidBodySet,
    current_tick: GameTickNumber,
) {
    match combatant_object.combatant_state.clone() {
        crate::game_objects::combatant::CombatantState::Idle => process_idle(combatant_object, rigid_body_set, current_tick),
        crate::game_objects::combatant::CombatantState::MovingToBall { ball_id } => process_moving_to_ball(combatant_object, ball_id, rigid_body_set, current_tick),
        crate::game_objects::combatant::CombatantState::MovingToPlate { plate_id } => process_moving_to_plate(combatant_object, plate_id, rigid_body_set, current_tick),
        crate::game_objects::combatant::CombatantState::RecoilingFromExplosion {  } => {},
    }
}

fn process_idle(
    combatant_object: &mut CombatantObject,
    rigid_body_set: &RigidBodySet,
    current_tick: GameTickNumber
) {
    // ZJ-TODO: pick something based on combatant stats
    // When idle, we'll simply pick a random thing to do        
    let combatant_rb = rigid_body_set
        .get(combatant_object.rigid_body_handle().expect("failed to get combatant rigid body handle"))
        .expect("failed to get combatant rigid body");

    let _combatant_pos = combatant_rb.translation();

    // ZJ-TODO: get nearest plate to combatant, remove hardcoded constant
    let temphardcode_nearest_plate_id = 1;

    if combatant_object.id == 1 && combatant_object.plate().is_none() {
        combatant_object.change_state(current_tick, CombatantState::MovingToPlate { plate_id: temphardcode_nearest_plate_id });
    }
}

fn process_moving_to_ball(
    combatant_object: &mut CombatantObject,
    ball_id: BallId,
    rigid_body_set: &RigidBodySet,
    current_tick: GameTickNumber
) {
    let Some(current_ball_id) = combatant_object.ball() else {
        return;
    };

    // When we've pathed to our desired ball, set our state back to Idle and re-run the AI logic for this combatant
    if current_ball_id == ball_id {
        combatant_object.pickup_ball(ball_id);
        combatant_object.change_state(current_tick, CombatantState::Idle);
        process_combatant(combatant_object, rigid_body_set, current_tick);
        return;
    }
}

fn process_moving_to_plate(
    combatant_object: &mut CombatantObject,
    plate_id: PlateId,
    rigid_body_set: &RigidBodySet,
    current_tick: GameTickNumber
) {
    let Some(current_plate_id) = combatant_object.plate() else {
        return;
    };

    // When we've pathed to our desired plate, set our state back to Idle and re-run the AI logic for this combatant
    if current_plate_id == plate_id {
        combatant_object.change_state(current_tick, CombatantState::Idle);
        process_combatant(combatant_object, rigid_body_set, current_tick);
        return;
    }
}