use crate::{ai::{belief::Belief, goal::GoalBuilder}, game_objects::combatant::CombatantObject, game_state::GameState};

use super::goal::Goal;

pub fn idle_goal() -> Goal {
    GoalBuilder::new()
        .name("Idle")
        .build()
}

pub fn goals(_combatant_object: &CombatantObject, _game_state: &GameState) -> Vec<Goal> {
    let mut goals = vec![
        GoalBuilder::new()
            .name("Score Points")
            .desired_beliefs(vec![Belief::SelfOnPlate])
            .priority(10)
            .build(),
        idle_goal()
    ];

    // Note: comparing b's priority to a (instead of comparing a's priority to b) as we want the largest priority goals first
    goals.sort_by(|a, b| b.priority().partial_cmp(&a.priority()).unwrap());

    goals
}
