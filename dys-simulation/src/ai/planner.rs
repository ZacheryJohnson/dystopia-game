use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use dys_satisfiable::SatisfiabilityTest;
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
    make_plan(agent, game_state, goals, actions).plan()
}

// fn make_plan(
//     agent: &impl Agent,
//     game_state: Arc<Mutex<GameState>>,
//     goals: Vec<Goal>,
//     actions: Vec<Action>,
// ) -> Vec<Action> {
//     tracing::debug!("Planning for combatant {} with beliefs {:?}", agent.combatant().id, agent.beliefs());
//
//     for goal in next_best_goal(agent, game_state, goals) {
//         tracing::debug!("Considering goal {}", goal.name());
//
//         let mut action_plan: Vec<Action> = vec![];
//         let mut desired_beliefs_remaining = goal.desired_beliefs();
//
//         while let Some(desired_belief) = desired_beliefs_remaining.pop() {
//             let Some(action) = get_action_for_belief(&desired_belief, &agent.beliefs(), &actions) else {
//                 action_plan.clear();
//                 desired_beliefs_remaining.push(desired_belief);
//                 break;
//             };
//
//             tracing::debug!("Selecting action {}", action.name());
//             for newly_desired_belief in action.prerequisite_beliefs() {
//                 if !agent.beliefs().can_satisfy(newly_desired_belief) {
//                     desired_beliefs_remaining.push(newly_desired_belief.to_owned());
//                     tracing::debug!("Adding new desired belief {:?}", newly_desired_belief);
//                 }
//             }
//             action_plan.push(action.to_owned());
//         }
//
//         if !desired_beliefs_remaining.is_empty() && action_plan.is_empty() {
//             tracing::debug!("failed to get action plan for goal {}; trying next goal", goal.name());
//             continue;
//         }
//
//         // Validate the plan we've just created
//         let mut agent_beliefs = agent.beliefs();
//         let mut can_perform = true;
//         for action in action_plan.iter().rev() {
//             if !action.can_perform(&agent_beliefs) {
//                 tracing::debug!("Failed to perform action {} in new plan; trying next goal", action.name());
//                 can_perform = false;
//                 break;
//             }
//
//             agent_beliefs = beliefs_if_completed(&action, agent_beliefs);
//         }
//
//         if !can_perform {
//             action_plan.clear();
//             continue;
//         }
//
//         tracing::debug!("Returning action plan: {:?}", action_plan);
//
//         return action_plan
//             .into_iter()
//             .map(|action| action.to_owned())
//             .collect();
//     }
//
//     vec![]
// }

#[derive(Clone)]
struct Plan {
    actions: Vec<Action>,
    dbg_str: String,
}

impl Plan {
    pub fn new() -> Self {
        Plan { actions: vec![], dbg_str: String::new() }
    }

    pub fn from(actions: &[Action]) -> Self {
        let mut new_plan = Plan { actions: actions.to_vec(), dbg_str: String::new() };
        new_plan.dbg_str = format!("{:?}", new_plan);
        new_plan
    }

    pub fn push(&mut self, action: Action) {
        self.actions.push(action);
        self.dbg_str = format!("{:?}", self);
    }

    pub fn extend(&mut self, actions: &[Action]) {
        self.actions.extend(actions.to_vec());
        self.dbg_str = format!("{:?}", self);
    }

    pub fn plan(&self) -> Vec<Action> {
        self.actions.clone()
    }
}

impl Debug for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] ", self.actions.iter().map(|action| action.cost()).sum::<f32>())?;

        for action in &self.actions {
            write!(f, "({})<-", action.name())?;
        }

        write!(f, "(start)")
    }
}

#[derive(Clone, Debug)]
struct PlannerState<'a> {
    lifetime: u8,
    pub actions: &'a Vec<Action>,
    pub action_plan: Plan,
    pub belief_set: BeliefSet,
    pub current_desired_belief: BeliefTest,
    pub remaining_desired_beliefs: Vec<BeliefTest>,
    pub prohibited_beliefs: Vec<BeliefTest>,
}

impl<'a> PlannerState<'a> {
    pub fn new(
        lifetime: u8,
        actions: &'a Vec<Action>,
        belief_set: BeliefSet,
        current_desired_belief: BeliefTest,
        remaining_desired_beliefs: Vec<BeliefTest>,
    ) -> Self {
        PlannerState {
            lifetime,
            actions,
            action_plan: Plan::new(),
            belief_set,
            current_desired_belief,
            remaining_desired_beliefs,
            prohibited_beliefs: vec![],
        }
    }

    pub fn new_after_completing(&self, action: &Action) -> Option<PlannerState> {
        // If our lifetime is zero, we've reached our maximum action plan length
        if self.lifetime == 0 {
            return None;
        }

        // If we had prohibited beliefs from previously planned actions, abort
        for prohibited_belief in &self.prohibited_beliefs {
            for action_completion_belief in action.completion_beliefs() {
                if prohibited_belief.satisfied_by(action_completion_belief.to_owned()) {
                    return None;
                }
            }

            for action_promised_belief in action.promised_beliefs() {
                if prohibited_belief.satisfied_by(action_promised_belief.to_owned()) {
                    return None;
                }
            }
        }

        let mut new_planner_state = self.clone();
        new_planner_state.lifetime = self.lifetime - 1;

        for prerequisite_belief in action.prerequisite_beliefs() {
            if !new_planner_state.belief_set.can_satisfy(prerequisite_belief) {
                new_planner_state.remaining_desired_beliefs.push(prerequisite_belief.to_owned());
            }
        }

        for prohibited_belief in action.prohibited_beliefs() {
            new_planner_state.prohibited_beliefs.push(prohibited_belief.to_owned());
        }

        for consumed_belief in action.consumed_beliefs() {
            new_planner_state.belief_set.remove_beliefs_by_test(consumed_belief);
        }

        new_planner_state.belief_set.add_beliefs(action.completion_beliefs());
        new_planner_state.belief_set.add_beliefs(action.promised_beliefs());
        new_planner_state.action_plan.push(action.to_owned());

        Some(new_planner_state)
    }
}

fn make_plan(
    agent: &impl Agent,
    game_state: Arc<Mutex<GameState>>,
    goals: Vec<Goal>,
    actions: Vec<Action>,
) -> Plan {
    for goal in next_best_goal(agent, game_state, goals) {
        let mut desired_beliefs_remaining = goal.desired_beliefs();
        let Some(next_desired_belief) = desired_beliefs_remaining.pop() else {
            tracing::error!("Considering goal {} where we already have all desired beliefs!", goal.name());
            continue;
        };

        let planner_state = PlannerState::new(5, &actions, agent.beliefs(), next_desired_belief, desired_beliefs_remaining);
        let potential_plans = make_potential_plans(planner_state);

        // Validate our potential plans from front to back
        let retain_fn = |plan: &Plan| -> bool {
            let mut plan = plan.plan();
            plan.reverse();

            let mut beliefs = agent.beliefs();
            for action in plan {
                for prohibited_belief in action.prohibited_beliefs() {
                    if beliefs.can_satisfy(prohibited_belief) {
                        return false;
                    }
                }

                for prerequisite_belief in action.prerequisite_beliefs() {
                    if !beliefs.can_satisfy(prerequisite_belief) {
                        return false;
                    }
                }

                for consumed_belief in action.consumed_beliefs() {
                    beliefs.remove_beliefs_by_test(consumed_belief);
                }

                beliefs.add_beliefs(action.completion_beliefs());
                beliefs.add_beliefs(action.promised_beliefs());
            }

            true
        };

        let mut potential_plans = potential_plans
            .into_iter()
            .filter(retain_fn)
            .collect::<Vec<_>>();

        if potential_plans.is_empty() {
            continue;
        }
        potential_plans.sort_by_key(|potential_plan| {
            // Because floats are not Ord, we'll cast to ints
            // I don't love this, but at worst our chosen plan is <0.5 cost worse than the next best
            potential_plan
                .plan()
                .iter()
                .map(|action| action.cost().round() as u32)
                .sum::<u32>()
        });

        return potential_plans.first().unwrap().to_owned();
    }

    tracing::warn!("Failed to construct plan for any goal - this is very bad!");
    Plan::new()
}

fn make_potential_plans(
    planner_state: PlannerState,
) -> Vec<Plan> {
    let mut potential_plans: Vec<Plan> = vec![];

    let filtered_actions = planner_state.actions.iter().filter(|action| {
        let meets_with_completion_belief = action
            .completion_beliefs()
            .iter()
            .any(|belief| planner_state.current_desired_belief.satisfied_by(belief.to_owned()));

        let meets_with_promised_belief = action
            .promised_beliefs()
            .iter()
            .any(|belief| planner_state.current_desired_belief.satisfied_by(belief.to_owned()));

        meets_with_completion_belief || meets_with_promised_belief
    }).collect::<Vec<_>>();

    for action in filtered_actions {
        let Some(mut updated_planner_state) = planner_state.new_after_completing(action) else {
            // This operation only fails if we have reached the maximum lifetime (eg action depth)
            break;
        };

        if let Some(next_needed_belief) = updated_planner_state.remaining_desired_beliefs.pop() {
            updated_planner_state.current_desired_belief = next_needed_belief;
            for plan in make_potential_plans(updated_planner_state) {
                if plan.plan().is_empty() {
                    break;
                }

                let mut new_plan = Plan::from(&[action.to_owned()]);
                new_plan.extend(&plan.plan());
                potential_plans.push(new_plan);
            }
        } else {
            potential_plans.push(Plan::from(&[action.to_owned()]));
        }
    }

    potential_plans
}

#[tracing::instrument(name = "planner::get_action_for_belief", skip_all, level = "trace")]
fn get_action_for_belief<'a>(
    desired_belief: &BeliefTest,
    _: &BeliefSet,
    actions: &'a [Action]
) -> Option<&'a Action> {
    actions
        .iter()
        .find(|action| action.can_satisfy(desired_belief.clone()))
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
                    .any(|desired_belief| !agent_beliefs.can_satisfy(desired_belief))
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
    use dys_satisfiable::SatisfiableField;
    use crate::{ai::{belief::Belief, goal::GoalBuilder, planner::next_best_goal, test_utils::TestAgent}};
    use crate::ai::agent::Agent;
    use crate::ai::belief::SatisfiableBelief;

    fn make_test_agent() -> TestAgent {
        TestAgent::new()
    }

    mod plan {
        use dys_satisfiable::SatisfiableField;
        use crate::ai::action::ActionBuilder;
        use crate::ai::belief::{Belief, SatisfiableBelief};
        use crate::ai::goal::GoalBuilder;
        use crate::ai::planner;
        use crate::ai::planner::tests::make_test_agent;
        use crate::ai::test_utils::make_test_game_state;

        #[test]
        fn big_boi_test() {
            let actions = vec![
                ActionBuilder::new()
                    .name("A")
                    .cost(1.0)
                    .completion(vec![
                        Belief::OnPlate { plate_id: 1, combatant_id: 1 },
                        Belief::BallIsFlying { ball_id: 1 }
                    ])
                    .build(),
                ActionBuilder::new()
                    .name("B")
                    .cost(1.0)
                    .requires(
                        SatisfiableBelief::OnPlate()
                            .plate_id(SatisfiableField::Exactly(1))
                            .combatant_id(SatisfiableField::Exactly(1))
                    )
                    .completion(vec![
                        Belief::HeldBall { ball_id: 1, combatant_id: 1 }
                    ])
                    .build(),
                ActionBuilder::new()
                    .name("C")
                    .cost(1.0)
                    .requires(
                        SatisfiableBelief::HeldBall()
                            .ball_id(SatisfiableField::Exactly(1))
                            .combatant_id(SatisfiableField::Exactly(1))
                    )
                    .promises(
                        Belief::DirectLineOfSightToCombatant { self_combatant_id: 1, other_combatant_id: 2 }
                    )
                    .build(),
                ActionBuilder::new()
                    .name("D")
                    .cost(0.0)
                    .prohibits(
                        SatisfiableBelief::OnPlate()
                            .combatant_id(SatisfiableField::Exactly(1))
                            .plate_id(SatisfiableField::Exactly(1))
                    )
                    .requires(
                        SatisfiableBelief::BallIsFlying()
                            .ball_id(SatisfiableField::Exactly(1))
                    )
                    .completion(vec![
                        Belief::DirectLineOfSightToCombatant { self_combatant_id: 1, other_combatant_id: 2 }
                    ])
                    .build()
            ];

            let goals = vec![
                GoalBuilder::new()
                    .name("Do C")
                    .priority(1)
                    .desired_belief(
                        SatisfiableBelief::DirectLineOfSightToCombatant()
                            .self_combatant_id(SatisfiableField::Exactly(1))
                            .other_combatant_id(SatisfiableField::Exactly(2))
                    )
                    .build()
            ];

            let agent = make_test_agent();
            let game_state = make_test_game_state(None);
            let plan = planner::make_plan(&agent, game_state, goals, actions);
            assert_eq!(plan.plan().len(), 3);
        }
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
