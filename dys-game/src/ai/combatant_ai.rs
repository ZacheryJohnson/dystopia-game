use rapier3d::dynamics::RigidBodySet;

use crate::{game_objects::{combatant::{CombatantObject, CombatantState}, game_object::GameObject}, game_tick::GameTickNumber};

pub fn process_combatant(
    combatant_object: &mut CombatantObject,
    rigid_body_set: &RigidBodySet,
    current_tick: GameTickNumber,
) {
    match combatant_object.combatant_state.clone() {
        crate::game_objects::combatant::CombatantState::Idle => process_idle(combatant_object, rigid_body_set, current_tick),
        crate::game_objects::combatant::CombatantState::MovingToBall { ball_id } => {},
        crate::game_objects::combatant::CombatantState::MovingToPlate { plate_id } => {},
        crate::game_objects::combatant::CombatantState::RecoilingFromExplosion {  } => {},
    }
}

fn process_idle(
    combatant_object: &mut CombatantObject,
    rigid_body_set: &RigidBodySet,
    current_tick: GameTickNumber)
{
    // ZJ-TODO: pick something based on combatant stats
    // When idle, we'll simply pick a random thing to do        
    let combatant_rb = rigid_body_set
        .get(combatant_object.rigid_body_handle().expect("failed to get combatant rigid body handle"))
        .expect("failed to get combatant rigid body");

    let _combatant_pos = combatant_rb.translation();

    // ZJ-TODO: get nearest plate to combatant, remove hardcoded constant
    let temphardcode_nearest_plate_id = 1;

    if combatant_object.id == 1 && !combatant_object.on_plate() {
        combatant_object.change_state(current_tick, CombatantState::MovingToPlate { plate_id: temphardcode_nearest_plate_id });
    }
}