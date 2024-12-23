use std::sync::{Arc, Mutex};
use crate::{ai::{belief::Belief, goal::GoalBuilder}, game_objects::combatant::CombatantObject, game_state::GameState};

use super::goal::Goal;

pub fn idle_goal() -> Goal {
    GoalBuilder::new()
        .name("Idle")
        .build()
}

pub fn goals(
    _combatant_object: &CombatantObject,
    _game_state: Arc<Mutex<GameState>>,
) -> Vec<Goal> {
    vec![
        GoalBuilder::new()
            .name("Score Points")
            .desired_beliefs(vec![Belief::SelfOnPlate])
            .priority(10)
            .build(),
        GoalBuilder::new()
            .name("Hold Ball")
            .desired_beliefs(vec![Belief::SelfHasBall])
            .priority(10)
            .build(),
        idle_goal()
    ]
}
