use std::sync::{Arc, Mutex};
use dys_satisfiable::SatisfiableField;
use crate::{ai::goal::GoalBuilder, game_objects::combatant::CombatantObject, game_state::GameState};
use crate::ai::belief::SatisfiableBelief;
use super::goal::Goal;

pub fn idle_goal() -> Goal {
    GoalBuilder::new()
        .name("Idle")
        .build()
}

pub fn goals(
    combatant_object: &CombatantObject,
    _game_state: Arc<Mutex<GameState>>,
) -> Vec<Goal> {
    vec![
        GoalBuilder::new()
            .name("Score Points")
            .desired_beliefs(vec![
                SatisfiableBelief::OnPlate()
                    .combatant_id(SatisfiableField::Exactly(combatant_object.id))
            ])
            .priority(10)
            .build(),
        GoalBuilder::new()
            .name("Hold Ball")
            .desired_beliefs(vec![
                SatisfiableBelief::HeldBall()
                    .combatant_id(SatisfiableField::Exactly(combatant_object.id))
            ])
            .priority(10)
            .build(),
        idle_goal()
    ]
}
