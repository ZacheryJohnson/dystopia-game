use crate::{ai::goap::goals::goals, game_objects::combatant::CombatantObject, game_state::GameState};

use super::{action::Action, actions::actions, agent::Agent};

pub(super) struct Planner;

impl Planner {
    pub fn plan(agent: &Agent, combatant: &CombatantObject, game_state: &GameState) -> Vec<Action> {
        // Pick a goal
        // ZJ-TODO: don't always pick this goal
        let goals = goals();
        let goal = goals.first().unwrap();

        // Determine actions to get to goal
        let mut action_plan = vec![];
        let mut desired_beliefs_remaining = goal.desired_beliefs();
        while let Some(desired_belief) = desired_beliefs_remaining.pop() {
            let actions = actions();
            let mut potential_actions: Vec<Action> = actions.into_iter().filter(|action| {
                action.completion_beliefs().contains(&desired_belief) && action.can_perform(agent.beliefs())
            }).collect();

            let Some(action) = potential_actions.pop() else {
                tracing::warn!("failed to get potential action for desired belief {:?}", desired_belief);
                break;
            };

            tracing::debug!("Adding action {} to plan", action.name());
            action_plan.push(action);
        }

        action_plan
    }
}