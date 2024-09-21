use crate::ai::{belief::Belief, goal::GoalBuilder};

use super::goal::Goal;

pub fn goals() -> Vec<Goal> {
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