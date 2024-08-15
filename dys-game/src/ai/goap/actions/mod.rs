use rapier3d::prelude::*;

use crate::ai::goap::{action::ActionBuilder, belief::Belief, strategies::move_to_location::MoveToLocationStrategy};

use super::action::Action;

pub(in crate::ai::goap)
fn actions() -> Vec<Action> {
    vec![
        ActionBuilder::new()
            .name("Move to Plate")
            .strategy(Box::new(MoveToLocationStrategy::new(point![50.0, 0.0, 50.0])))
            .completion(vec![Belief::SelfOnPlate])
            .build(),
    ]
}