use std::collections::HashMap;

use dys_world::arena::plate::PlateId;
use ordered_float::OrderedFloat;
use rand::Rng;
use rapier3d::{dynamics::RigidBodySet, geometry::ColliderSet};

use crate::{game_objects::{ball::{BallId, BallObject}, combatant::{CombatantObject, CombatantState}, game_object::GameObject, plate::PlateObject}, game_tick::GameTickNumber};

struct AiGameState<'gs> {
    rigid_body_set: &'gs RigidBodySet,
    collider_set: &'gs ColliderSet,
    plates: &'gs mut HashMap<PlateId, PlateObject>,
    balls: &'gs mut HashMap<BallId, BallObject>,
    current_tick: GameTickNumber, 
}

impl<'gs> AiGameState<'gs> {
    pub fn new(
        rigid_body_set: &'gs RigidBodySet,
        collider_set: &'gs ColliderSet,
        plates: &'gs mut HashMap<PlateId, PlateObject>,
        balls: &'gs mut HashMap<BallId, BallObject>,
        current_tick: GameTickNumber
    ) -> AiGameState<'gs> {
        AiGameState {
            rigid_body_set,
            collider_set,
            plates,
            balls,
            current_tick,
        }
    }
}

pub fn process_combatant(
    combatant_object: &mut CombatantObject,
    rigid_body_set: &RigidBodySet,
    collider_set: &ColliderSet,
    plates: &mut HashMap<PlateId, PlateObject>,
    balls: &mut HashMap<BallId, BallObject>,
    current_tick: GameTickNumber,
) {
    let mut ai_game_state = AiGameState::new(rigid_body_set, collider_set, plates, balls, current_tick);

    process_combatant_internal(combatant_object, &mut ai_game_state)
}

fn process_combatant_internal(
    combatant_object: &mut CombatantObject,
    ai_game_state: &mut AiGameState,
) {
    match combatant_object.combatant_state.clone() {
        crate::game_objects::combatant::CombatantState::Idle => process_idle(combatant_object, ai_game_state),
        crate::game_objects::combatant::CombatantState::MovingToBall { ball_id } => process_moving_to_ball(combatant_object, ball_id, ai_game_state),
        crate::game_objects::combatant::CombatantState::MovingToPlate { plate_id } => process_moving_to_plate(combatant_object, plate_id, ai_game_state),
        crate::game_objects::combatant::CombatantState::RecoilingFromExplosion { } => {},
    }
}

fn process_idle(
    combatant_object: &mut CombatantObject,
    ai_game_state: &mut AiGameState,
) {
    // ZJ-TODO: pick something based on combatant stats
    // When idle, we'll simply pick a random thing to do

    // Biased coin flip: 70% move to a plate, 30% move to a ball

    if rand::thread_rng().gen_bool(0.7) {
        // Move to plate
        let combatant_rb = ai_game_state.rigid_body_set
            .get(combatant_object.rigid_body_handle().expect("failed to get combatant rigid body handle"))
            .expect("failed to get combatant rigid body");

        let combatant_pos = combatant_rb.translation();

        let maybe_nearest_plate_id = ai_game_state.plates
                .iter()
                .map(|(id, plate_object)| (id, ai_game_state.collider_set.get(plate_object.collider_handle().unwrap()).unwrap().translation()))
                .min_by_key(|(_, plate_pos)| OrderedFloat::from((*plate_pos - combatant_pos).magnitude()));

        let (nearest_plate_id, _) = maybe_nearest_plate_id.expect("failed to find nearest plate for combatant - shouldn't panic and should instead pick another action");

        if combatant_object.plate().is_none() {
            combatant_object.change_state(ai_game_state.current_tick, CombatantState::MovingToPlate { plate_id: *nearest_plate_id });
        }
    } else {
        // Move to ball
        let combatant_rb = ai_game_state.rigid_body_set
            .get(combatant_object.rigid_body_handle().expect("failed to get combatant rigid body handle"))
            .expect("failed to get combatant rigid body");

        let combatant_pos = combatant_rb.translation();

        let maybe_nearest_ball_id = ai_game_state.balls
                .iter()
                .filter(|(_, ball_object)| ball_object.held_by.is_none())
                .map(|(id, ball_object)| (id, ai_game_state.rigid_body_set.get(ball_object.rigid_body_handle().unwrap()).unwrap().translation()))
                .min_by_key(|(_, ball_pos)| OrderedFloat::from((*ball_pos - combatant_pos).magnitude()));

        let Some((nearest_ball_id, _)) = maybe_nearest_ball_id else {
            // ZJ-TODO: immediately trigger another attempt
            //          for now, we just waste a tick doing nothing
            return;
        };

        if combatant_object.plate().is_none() {
            combatant_object.change_state(ai_game_state.current_tick, CombatantState::MovingToBall { ball_id: *nearest_ball_id });
        }
    }
}

fn process_moving_to_ball(
    combatant_object: &mut CombatantObject,
    ball_id: BallId,
    ai_game_state: &mut AiGameState,
) {
    /*
        Future improvements:
        - determine if we have no chance of getting to a ball before someone else (teammate or opponent) and aborting if so
     */

    let Some(desired_ball_id) = combatant_object.ball() else {
        return;
    };

    // When we've pathed to our desired ball, set our state back to Idle and re-run the AI logic for this combatant
    if desired_ball_id == ball_id {
        combatant_object.change_state(ai_game_state.current_tick, CombatantState::Idle);
        process_combatant_internal(combatant_object, ai_game_state);
        return;
    }
}

fn process_moving_to_plate(
    combatant_object: &mut CombatantObject,
    plate_id: PlateId,
    ai_game_state: &mut AiGameState,
) {
    let Some(current_plate_id) = combatant_object.plate() else {
        return;
    };

    // When we've pathed to our desired plate, set our state back to Idle and re-run the AI logic for this combatant
    if current_plate_id == plate_id {
        combatant_object.change_state(ai_game_state.current_tick, CombatantState::Idle);
        process_combatant_internal(combatant_object, ai_game_state);
        return;
    }
}