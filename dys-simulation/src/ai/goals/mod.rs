use std::sync::{Arc, Mutex};
use dys_satisfiable::SatisfiableField;
use dys_world::attribute::attribute_type::AttributeType;
use crate::{ai::goal::GoalBuilder, game_objects::combatant::CombatantObject, game_state::GameState};
use crate::ai::belief::SatisfiableBelief;
use super::goal::Goal;

pub fn idle_goal() -> Goal {
    GoalBuilder::new()
        .name("Look Around")
        .desired_belief(SatisfiableBelief::ScannedEnvironment())
        .priority(1)
        .build()
}

pub fn goals(
    combatant_object: &CombatantObject,
    game_state: Arc<Mutex<GameState>>,
) -> Vec<Goal> {
    let combatant_instance = combatant_object.combatant.lock().unwrap();

    let attr = |attribute_type: AttributeType| {
        combatant_instance.get_attribute_value(&attribute_type).unwrap_or_default().floor() as u32
    };

    let teammate_ids = game_state.lock().unwrap().team_combatants(combatant_object.team)
        .iter()
        .map(|combatant_object| combatant_object.id)
        .collect::<Vec<_>>();

    // ZJ-TODO: refactor, goodness
    vec![
        GoalBuilder::new()
            .name("Score Points")
            .desired_beliefs(vec![
                SatisfiableBelief::OnPlate()
                    .combatant_id(SatisfiableField::Exactly(combatant_object.id))
            ])
            .priority(attr(AttributeType::Dexterity))
            .build(),
        GoalBuilder::new()
            .name("Throw Ball At Enemies")
            .desired_beliefs(vec![
                SatisfiableBelief::BallThrownAtCombatant()
                    .target_id(SatisfiableField::NotIn(teammate_ids.clone()))
            ])
            .priority(attr(AttributeType::Coordination) + attr(AttributeType::Strength))
            .build(),
        GoalBuilder::new()
            .name("Shove Combatants")
            .desired_belief(
                SatisfiableBelief::CombatantShoved()
                    .combatant_id(SatisfiableField::NotIn(teammate_ids.clone()))
            )
            .priority(attr(AttributeType::Constitution) + attr(AttributeType::Presence))
            .build(),
        // ZJ-TODO: goal: recover from explosion / self is cogent
        idle_goal()
    ]
}
