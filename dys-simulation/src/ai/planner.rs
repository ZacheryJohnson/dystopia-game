use std::sync::{Arc, Mutex};
use crate::{ai::goals::goals, game_state::GameState};
use crate::ai::belief::{BeliefSet, BeliefTest};
use super::{action::Action, actions::actions, agent::Agent, goal::Goal};

#[tracing::instrument(
    skip_all,
    level = "trace",
    fields(
        combatant_id = agent.combatant().id,
        tick = game_state.lock().unwrap().current_tick
    )
)]
pub fn plan(
    agent: &impl Agent,
    game_state: Arc<Mutex<GameState>>,
) -> Vec<Action> {
    let goals = goals(agent.combatant(), game_state.clone());
    let actions = actions(agent.combatant(), game_state.clone());
    make_plan(agent, game_state, goals, actions)
}

fn make_plan(
    agent: &impl Agent,
    game_state: Arc<Mutex<GameState>>,
    goals: Vec<Goal>,
    actions: Vec<Action>,
) -> Vec<Action> {
    tracing::debug!("Planning for combatant {} with beliefs {:?}", agent.combatant().id, agent.beliefs());

    #[cfg(debug_assertions)]
    {
        // Useful for debugging, but unnecessary in shipping builds
        let combatant_id = agent.combatant().id;
        let current_tick = game_state.lock().unwrap().current_tick.to_owned();
    }

    for goal in next_best_goal(agent, game_state, goals) {
        tracing::debug!("Considering goal {}", goal.name());

        let mut action_plan: Vec<Action> = vec![];
        let mut desired_beliefs_remaining = goal.desired_beliefs();

        while let Some(desired_belief) = desired_beliefs_remaining.pop() {
            let Some(action) = get_action_for_belief(&desired_belief, &agent.beliefs(), &actions) else {
                action_plan.clear();
                desired_beliefs_remaining.push(desired_belief);
                break;
            };

            tracing::debug!("Selecting action {}", action.name());
            for newly_desired_belief in action.prerequisite_beliefs() {
                if !agent.beliefs().can_satisfy(newly_desired_belief.to_owned()) {
                    desired_beliefs_remaining.push(newly_desired_belief.to_owned());
                    tracing::debug!("Adding new desired belief {:?}", newly_desired_belief);
                }
            }
            action_plan.push(action.to_owned());
        }

        if !desired_beliefs_remaining.is_empty() && action_plan.is_empty() {
            tracing::debug!("failed to get action plan for goal {}; trying next goal", goal.name());
            continue;
        }

        tracing::debug!("Returning action plan: {:?}", action_plan);

        return action_plan
            .into_iter()
            .map(|action| action.to_owned())
            .collect();
    }

    vec![]
}

#[tracing::instrument(name = "planner::get_action_for_belief", skip_all, level = "trace")]
fn get_action_for_belief<'a>(
    desired_belief: &BeliefTest,
    _: &BeliefSet,
    actions: &'a [Action]
) -> Option<&'a Action> {
    actions
        .iter()
        .filter(|action| action.can_satisfy(desired_belief.clone()))
        .next()
}

/// The best goal is the highest priority goal where the agent doesn't already have all of the desired beliefs.
/// In the event we can't find a good goal from the goals provided, we'll return the Idle goal.
#[tracing::instrument(fields(combatant_id = agent.combatant().id), skip_all, level = "debug")]
fn next_best_goal<'a>(
    agent: &impl Agent,
    _: Arc<Mutex<GameState>>,
    goals: Vec<Goal>,
) -> impl Iterator<Item = Goal> {
    let agent_beliefs = agent.beliefs();
    let mut goals: Vec<_> = goals
        .into_iter()
        .filter(|goal| {
            let desired_beliefs = goal.desired_beliefs();
            if desired_beliefs.is_empty() {
                true
            } else {
                desired_beliefs
                    .iter()
                    .any(|desired_belief| !agent_beliefs.can_satisfy(desired_belief.to_owned()))
            }
        })
        .collect();

    // Note: comparing b's priority to a (instead of comparing a's priority to b) as we want the largest priority goals first
    goals.sort_by(|a, b| b.priority().partial_cmp(&a.priority()).unwrap());

    tracing::debug!("Prioritized goals: {:?}", goals);

    goals.into_iter()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::{Arc, Mutex}};

    use dys_world::{arena::{navmesh::{ArenaNavmesh, ArenaNavmeshConfig}, Arena}, schedule::{calendar::{Date, Month}, schedule_game::ScheduleGame}, team::instance::TeamInstance};
    use rand::SeedableRng;
    use rand_pcg::Pcg64;
    use dys_satisfiable::SatisfiableField;
    use crate::{ai::{belief::Belief, goal::GoalBuilder, planner::next_best_goal, test_utils::{self, TestAgent}}, game::Game, game_state::GameState, physics_sim::PhysicsSim, simulation::config::SimulationConfig};
    use crate::ai::agent::Agent;
    use crate::ai::belief::SatisfiableBelief;
    use crate::game_state::{BallsMapT, CollidersMapT, CombatantsMapT, PlatesMapT};

    fn make_test_agent() -> TestAgent {
        test_utils::TestAgent::new()
    }

    mod next_best_goal {
        use crate::ai::test_utils::make_test_game_state;
        use super::*;

        #[test]
        fn test_no_goals_receive_idle_goal() {
            let agent = make_test_agent();
            let game_state = make_test_game_state(None);

            let no_goals = vec![];

            let best_goal = next_best_goal(&agent, game_state, no_goals).next();

            assert!(best_goal.is_none());
        }

        #[test]
        fn test_best_goal_does_not_have_desired_beliefs_isnt_idle_goal() {
            let agent = make_test_agent();
            let game_state = make_test_game_state(None);

            let expected_goal_name = "TestGoalSelfOnPlate";

            let goals = vec![
                GoalBuilder::new()
                    .name(expected_goal_name)
                    .priority(1)
                    .desired_beliefs(vec![SatisfiableBelief::OnPlate()])
                    .build()
            ];

            let best_goal = next_best_goal(&agent, game_state, goals).next().unwrap();

            assert_eq!(expected_goal_name, best_goal.name());
        }

        #[test]
        fn test_best_goal_has_desired_beliefs_is_idle_goal() {
            let mut agent = make_test_agent();
            let game_state = make_test_game_state(None);

            let expected_goal_name = "TestGoalSelfOnPlate";

            agent.set_beliefs(vec![
                Belief::OnPlate { combatant_id: agent.combatant().id, plate_id: 1 }
            ]);

            let goals = vec![
                GoalBuilder::new()
                    .name(expected_goal_name)
                    .priority(1)
                    .desired_beliefs(vec![
                        SatisfiableBelief::OnPlate()
                            .combatant_id(SatisfiableField::Exactly(agent.combatant().id))
                    ])
                    .build()
            ];

            let best_goal = next_best_goal(&agent, game_state, goals).next();

            assert!(best_goal.is_none());
        }

        #[test]
        fn test_best_goal_higher_priority_wins() {
            let agent = make_test_agent();
            let game_state = make_test_game_state(None);

            let lower_priority_goal_name = "TestLowerPriorityGoal";
            let higher_priority_goal_name = "TestHigherPriorityGoal";

            let goals = vec![
                GoalBuilder::new()
                    .name(lower_priority_goal_name)
                    .priority(1)
                    .desired_beliefs(vec![
                        SatisfiableBelief::OnPlate()
                            .combatant_id(SatisfiableField::Exactly(agent.combatant().id))
                    ])
                    .build(),
                GoalBuilder::new()
                    .name(higher_priority_goal_name)
                    .priority(2)
                    .desired_beliefs(vec![
                        SatisfiableBelief::HeldBall()
                            .combatant_id(SatisfiableField::Exactly(agent.combatant().id))
                    ])
                    .build(),
            ];

            let best_goal = next_best_goal(&agent, game_state, goals).next().unwrap();

            assert_eq!(higher_priority_goal_name, best_goal.name());
        }
    }
}
