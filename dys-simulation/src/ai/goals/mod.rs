use std::sync::{Arc, Mutex};
use dys_satisfiable::SatisfiableField;
use dys_world::attribute::attribute_type::AttributeType;
use crate::{ai::goal::GoalBuilder, game_objects::combatant::CombatantObject, game_state::GameState};
use crate::ai::belief::SatisfiableBelief;
use super::goal::Goal;

// ZJ-TODO: move to config
const ON_PLATE_PRIORITY_MULTIPLIER: f32 = 4.5;

pub fn idle_goal() -> Goal {
    GoalBuilder::new()
        .name("Look Around")
        .desired_belief(SatisfiableBelief::ScannedEnvironment())
        .priority(1.0)
        .build()
}

pub fn goals(
    combatant_object: &CombatantObject,
    game_state: Arc<Mutex<GameState>>,
) -> Vec<Goal> {
    let combatant_instance = combatant_object.combatant.lock().unwrap();

    let attr = |attribute_type: AttributeType| {
        combatant_instance.get_attribute_value(&attribute_type).unwrap_or_default()
    };

    let teammate_ids = game_state.lock().unwrap().team_combatants(combatant_object.team)
        .iter()
        .map(|combatant_object| combatant_object.id)
        .collect::<Vec<_>>();

    // ZJ-TODO: refactor, goodness
    vec![
        GoalBuilder::new()
            .name("Score Points")
            .desired_belief(
                SatisfiableBelief::OnPlate()
                    .combatant_id(SatisfiableField::Exactly(combatant_object.id))
            )
            .priority(0.5 * attr(AttributeType::Dexterity))
            .repeatable(true)
            .build(),
        GoalBuilder::new()
            .name("Throw Ball At Enemies On Plates")
            .desired_belief(
                SatisfiableBelief::BallThrownAtCombatant()
                    .target_combatant_id(SatisfiableField::NotIn(teammate_ids.clone()))
                    .target_on_plate(SatisfiableField::NotExactly(None))
            )
            .priority(ON_PLATE_PRIORITY_MULTIPLIER * (attr(AttributeType::Coordination) + attr(AttributeType::Strength)))
            .build(),
        GoalBuilder::new()
            .name("Throw Ball At Enemies Off Plates")
            .desired_belief(
                SatisfiableBelief::BallThrownAtCombatant()
                    .target_combatant_id(SatisfiableField::NotIn(teammate_ids.clone()))
                    .target_on_plate(SatisfiableField::Exactly(None))
            )
            .priority(attr(AttributeType::Coordination) + attr(AttributeType::Strength))
            .build(),
        GoalBuilder::new()
            .name("Shove Combatants On Plates")
            .desired_belief(
                SatisfiableBelief::CombatantShoved()
                    .combatant_id(SatisfiableField::NotIn(teammate_ids.clone()))
                    .on_plate(SatisfiableField::NotExactly(None))
            )
            .priority(ON_PLATE_PRIORITY_MULTIPLIER * (attr(AttributeType::Constitution) + attr(AttributeType::Presence)))
            .build(),
        GoalBuilder::new()
            .name("Shove Combatants Off Plates")
            .desired_belief(
                SatisfiableBelief::CombatantShoved()
                    .combatant_id(SatisfiableField::NotIn(teammate_ids.clone()))
                    .on_plate(SatisfiableField::Exactly(None))
            )
            .priority(attr(AttributeType::Constitution) + attr(AttributeType::Presence))
            .build(),
        GoalBuilder::new()
            .name("Catch Ball")
            .desired_belief(
                SatisfiableBelief::BallCaught()
                    .combatant_id(SatisfiableField::Exactly(combatant_object.id))
            )
            .priority(100.0)
            .build(),
        idle_goal()
    ]
}
