use crate::ai::goap::{belief::Belief, goal::GoalBuilder};

use super::goal::Goal;

pub(in crate::ai::goap)
fn goals() -> Vec<Goal> {
    vec![
        GoalBuilder::new()
            .name("Score Points")
            .desired_beliefs(vec![Belief::SelfOnPlate])
            .build(),
        GoalBuilder::new()
            .name("Idle")
            .build(),
    ]
}