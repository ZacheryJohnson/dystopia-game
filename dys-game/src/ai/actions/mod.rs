use std::sync::{Arc, Mutex};

use crate::{ai::{action::ActionBuilder, belief::Belief, strategies::move_to_location::MoveToLocationStrategy}, game_objects::{combatant::CombatantObject, game_object::GameObject}, game_state::GameState};

use super::action::Action;

/// ZJ-TODO: HACK: this value should be passed in through simulation settings.
/// This value allows us to make all movement actions cheaper/more expensive,
/// as other actions may have lower/higher absolute costs.
const MOVE_TO_LOCATION_WEIGHT_HARDCODE_HACK: f32 = 0.2_f32;

pub fn actions(combatant: &CombatantObject, game_state: &GameState) -> Vec<Action> {
    let mut actions = vec![];

    let (rigid_body_set, collider_set, _) = game_state.physics_sim.sets();
    let combatant_pos = rigid_body_set.get(combatant.rigid_body_handle).unwrap().translation();

    for (plate_id, plate_object) in &game_state.plates {
        let plate_location = collider_set.get(plate_object.collider_handle().unwrap()).unwrap().translation();
        actions.push(
            ActionBuilder::new()
                .name(format!("Move to Plate {plate_id}"))
                .strategy(Arc::new(Mutex::new(
                    MoveToLocationStrategy::new((*combatant_pos).into(), (*plate_location).into(), game_state))
                ))
                .cost(MOVE_TO_LOCATION_WEIGHT_HARDCODE_HACK * (plate_location - combatant_pos).magnitude() / combatant.combatant.lock().unwrap().move_speed())
                .completion(vec![Belief::SelfOnPlate])
                .build()
        );
    }

    actions
}