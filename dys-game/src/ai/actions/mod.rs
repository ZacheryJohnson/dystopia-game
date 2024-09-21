use std::sync::{Arc, Mutex};

use crate::{ai::{action::ActionBuilder, belief::Belief, strategies::move_to_location::MoveToLocationStrategy}, game_objects::{combatant::CombatantObject, game_object::GameObject}, game_state::GameState};

use super::action::Action;

pub fn actions(combatant: &CombatantObject, game_state: &GameState) -> Vec<Action> {
    let mut actions = vec![];

    let (_, collider_set, _) = game_state.physics_sim.sets();
    for (plate_id, plate_object) in &game_state.plates {
        let plate_location = collider_set.get(plate_object.collider_handle().unwrap()).unwrap().translation();
        actions.push(
            ActionBuilder::new()
                .name(format!("Move to Plate {plate_id}"))
                .strategy(Arc::new(Mutex::new(
                    MoveToLocationStrategy::new((*plate_location).into(), combatant.rigid_body_handle().unwrap()))
                ))
                .completion(vec![Belief::SelfOnPlate])
                .build()
        );
    }

    actions
}