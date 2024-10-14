use crate::{ai::goals::goals, game_state::GameState};

use super::{action::Action, actions::actions, agent::Agent, goal::Goal, goals::idle_goal};

#[tracing::instrument(
    skip_all,
    level = "trace",
    fields(
        combatant_id = agent.combatant().id,
        tick = game_state.current_tick,
    )
)]
pub fn plan(agent: &impl Agent, game_state: &GameState) -> Vec<Action> {
    // Pick a goal
    let all_goals = goals(agent.combatant(), game_state);
    let goal = get_best_goal(agent, game_state, all_goals);

    // Determine actions to get to goal
    let mut action_plan: Vec<Action> = vec![];
    let mut desired_beliefs_remaining = goal.desired_beliefs();
    
    tracing::trace!("Picked goal {} as best; need beliefs: {:?}", goal.name(), desired_beliefs_remaining);

    while let Some(desired_belief) = desired_beliefs_remaining.pop() {
        let actions = actions(agent.combatant(), game_state);

        tracing::trace!("Considering all available actions: {:?}", actions);

        let mut potential_actions: Vec<Action> = actions.into_iter().filter(|action| {
            action.completion_beliefs().iter().any(|belief| belief.is_a(&desired_belief))
                && !action.prohibited_beliefs().iter().any(|belief| agent.beliefs().contains(belief))
        }).collect();

        tracing::trace!("Considering potential actions: {:?}", potential_actions);

        let Some(action) = potential_actions.pop() else {
            tracing::warn!("failed to get potential action for desired belief {:?}", desired_belief);
            return vec![];
        };

        let newly_desired_beliefs = action
            .prerequisite_beliefs()
            .into_iter()
            .filter(|belief| !agent.beliefs().contains(belief))
            .map(|belief| belief.to_owned())
            .collect::<Vec<_>>();

        tracing::trace!("Adding prerequisite beliefs for action {}: {:?}", action.name(), newly_desired_beliefs);

        desired_beliefs_remaining.extend(newly_desired_beliefs);

        tracing::trace!("Adding action {} to plan", action.name());
        action_plan.push(action);
    }

    action_plan
}

/// The best goal is the highest priority goal where the agent doesn't already have all of the desired beliefs.
/// In the event we can't find a good goal from the goals provided, we'll return the Idle goal.
#[tracing::instrument(fields(combatant_id = agent.combatant().id), skip_all, level = "trace")]
fn get_best_goal<'a>(agent: &impl Agent, _: &GameState, all_goals: Vec<Goal>) -> Goal {
    let mut goals: Vec<_> = all_goals
        .into_iter()
        .filter(|goal| !goal.desired_beliefs().iter().all(|belief| agent.beliefs().contains(&belief)))
        .collect();

    // Note: comparing b's priority to a (instead of comparing a's priority to b) as we want the largest priority goals first
    goals.sort_by(|a, b| b.priority().partial_cmp(&a.priority()).unwrap());

    goals
        .into_iter()
        .next()
        .unwrap_or(idle_goal())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::{Arc, Mutex}};

    use dys_world::{arena::{navmesh::{ArenaNavmesh, ArenaNavmeshConfig}, Arena}, schedule::{calendar::{Date, Month}, schedule_game::ScheduleGame}, team::team::Team};
    use rand::SeedableRng;
    use rand_pcg::Pcg64;

    use crate::{ai::{belief::Belief, goal::GoalBuilder, planner::get_best_goal, test_utils::{self, TestAgent}}, game::Game, game_state::GameState, physics_sim::PhysicsSim, simulation::config::SimulationConfig};

    fn make_test_agent() -> TestAgent {
        test_utils::TestAgent::new()
    }

    fn make_test_game_state() -> GameState {
        let game = Game {
            schedule_game: ScheduleGame {
                away_team: Arc::new(Mutex::new(Team {
                    id: 1,
                    name: String::from("TestAwayTeam"),
                    combatants: vec![],
                })),
                home_team: Arc::new(Mutex::new(Team {
                    id: 2,
                    name: String::from("TestHomeTeam"),
                    combatants: vec![],
                })),
                arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())), // ZJ-TODO: don't use arena's default values
                date: Date(Month::Arguscorp, 1, 10000),
            },
        };
        let simulation_config = SimulationConfig::default();
        let arena_navmesh = ArenaNavmesh::new_from(game.schedule_game.arena.clone(), ArenaNavmeshConfig::default());

        GameState {
            game,
            seed: [0; 32],
            rng: Pcg64::from_seed([0; 32]),
            physics_sim: PhysicsSim::new(simulation_config.ticks_per_second()),
            combatants: HashMap::new(),
            balls: HashMap::new(),
            plates: HashMap::new(),
            active_colliders: HashMap::new(),
            home_points: 0,
            away_points: 0,
            current_tick: 0,
            simulation_config,
            arena_navmesh,
        }
    }

    #[test]
    fn test_no_goals_receive_idle_goal() {
        let agent = make_test_agent();
        let game_state = make_test_game_state();

        let no_goals = vec![];

        let best_goal = get_best_goal(&agent, &game_state, no_goals);

        assert_eq!(String::from("Idle"), best_goal.name());
    }

    #[test]
    fn test_best_goal_does_not_have_desired_beliefs_isnt_idle_goal() {
        let agent = make_test_agent();
        let game_state = make_test_game_state();

        let expected_goal_name = "TestGoalSelfOnPlate";

        let goals = vec![
            GoalBuilder::new()
                .name(expected_goal_name)
                .priority(1)
                .desired_beliefs(vec![Belief::SelfOnPlate])
                .build()
        ];

        
        let best_goal = get_best_goal(&agent, &game_state, goals);

        assert_eq!(expected_goal_name, best_goal.name());
    }

    #[test]
    fn test_best_goal_has_desired_beliefs_is_idle_goal() {
        let mut agent = make_test_agent();
        let game_state = make_test_game_state();

        let expected_goal_name = "TestGoalSelfOnPlate";

        agent.set_beliefs(vec![Belief::SelfOnPlate]);

        let goals = vec![
            GoalBuilder::new()
                .name(expected_goal_name)
                .priority(1)
                .desired_beliefs(vec![Belief::SelfOnPlate])
                .build()
        ];

        
        let best_goal = get_best_goal(&agent, &game_state, goals);

        assert_eq!(String::from("Idle"), best_goal.name());
    }

    #[test]
    fn test_best_goal_higher_priority_wins() {
        let agent = make_test_agent();
        let game_state = make_test_game_state();

        let lower_priority_goal_name = "TestLowerPriorityGoal";
        let higher_priority_goal_name = "TestHigherPriorityGoal";

        let goals = vec![
            GoalBuilder::new()
                .name(lower_priority_goal_name)
                .priority(1)
                .desired_beliefs(vec![Belief::SelfOnPlate])
                .build(),
            GoalBuilder::new()
                .name(higher_priority_goal_name)
                .priority(2)
                .desired_beliefs(vec![Belief::SelfHasBall])
                .build(),            
        ];

        
        let best_goal = get_best_goal(&agent, &game_state, goals);

        assert_eq!(higher_priority_goal_name, best_goal.name());
    }
}