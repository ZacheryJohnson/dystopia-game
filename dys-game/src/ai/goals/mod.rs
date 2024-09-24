use crate::{ai::{belief::Belief, goal::GoalBuilder}, game_objects::combatant::CombatantObject, game_state::GameState};

use super::goal::Goal;

pub fn idle_goal() -> Goal {
    GoalBuilder::new()
        .name("Idle")
        .build()
}

pub fn goals(_combatant_object: &CombatantObject, _game_state: &GameState) -> Vec<Goal> {
    vec![
        GoalBuilder::new()
            .name("Score Points")
            .desired_beliefs(vec![Belief::SelfOnPlate])
            .priority(10)
            .build(),
        idle_goal()
    ]
}
