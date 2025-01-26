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
    game_state: Arc<Mutex<GameState>>,
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
            .name("Throw Ball At Enemies")
            .desired_beliefs(vec![
                SatisfiableBelief::BallThrownAtCombatant()
                    .target_id(SatisfiableField::NotIn(
                        game_state.lock().unwrap().team_combatants(combatant_object.team)
                            .iter()
                            .map(|combatant_object| combatant_object.id)
                            .collect()
                    ))
            ])
            .priority(10)
            .build(),
        // ZJ-TODO: goal: recover from explosion / self is cogent
        idle_goal()
    ]
}
