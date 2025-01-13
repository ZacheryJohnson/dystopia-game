use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use crate::game_state::GameState;
use crate::simulation::simulation_event::SimulationEvent;

use super::agent::Agent;
use super::belief::{Belief, BeliefSatisfiabilityTest, BeliefSet, BeliefTest};
use super::strategies::noop::NoopStrategy;
use super::strategy::Strategy;

pub type StrategyT = Arc<Mutex<dyn Strategy>>;

#[derive(Clone)]
pub struct Action {
    /// Name of the action
    name: String,

    /// Cost of performing this action
    /// Arbitrary float value
    cost: f32,

    strategy: StrategyT,

    /// Belief tests required for the action to be taken
    prerequisite_beliefs: Vec<BeliefTest>,

    /// Belief tests that prevent the action from being taken
    prohibited_beliefs: Vec<BeliefTest>,

    /// Concrete beliefs applied once the action completes successfully
    completion_beliefs: Vec<Belief>,
}

impl Action {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn cost(&self) -> f32 {
        self.cost
    }

    pub fn can_perform(&self, owned_beliefs: &BeliefSet) -> bool {
        let all_prereqs = self
            .prerequisite_beliefs
            .iter()
            .all(|belief| owned_beliefs.can_satisfy(belief.to_owned())); // ZJ-TODO: allow passing a reference instead of cloning

        let none_prohibited = self
            .prohibited_beliefs
            .iter()
            .all(|belief| !owned_beliefs.can_satisfy(belief.to_owned())); // ZJ-TODO: allow passing a reference instead of cloning

        let can_perform_strategy = self
            .strategy
            .lock()
            .unwrap()
            .can_perform(owned_beliefs);

        tracing::debug!("[action::can_perform] {}: all_prereqs: {all_prereqs} | none_prohibited: {none_prohibited} | can_perform_strategy: {can_perform_strategy}", self.name);

        all_prereqs && none_prohibited && can_perform_strategy
    }

    pub fn is_complete(&self) -> bool {
        self.strategy.lock().unwrap().is_complete()
    }

    #[tracing::instrument(name = "action::tick", fields(combatant_id = agent.combatant().id), skip_all, level = "trace")]
    pub fn tick(
        &mut self,
        agent: &impl Agent,
        game_state: Arc<Mutex<GameState>>,
    ) -> Option<Vec<SimulationEvent>> {
        let mut strategy = self.strategy.lock().unwrap();
        if !strategy.can_perform(&agent.beliefs()) {
            tracing::debug!("Cannot perform action {}", self.name());
            return None;
        }

        strategy.tick(agent, game_state)
    }

    pub fn completion_beliefs(&self) -> &Vec<Belief> {
        &self.completion_beliefs
    }

    pub fn prohibited_beliefs(&self) -> &Vec<BeliefTest> {
        &self.prohibited_beliefs
    }

    pub fn prerequisite_beliefs(&self) -> &Vec<BeliefTest> {
        &self.prerequisite_beliefs
    }
}

impl Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("name", &self.name)
            .field("cost", &self.cost)
            .field("strategy", &self.strategy.lock().unwrap().name())
            .field("prerequisite_beliefs", &self.prerequisite_beliefs)
            .field("prohibited_beliefs", &self.prohibited_beliefs)
            .field("completion_beliefs", &self.completion_beliefs)
            .finish()
    }
}


pub(super) struct ActionBuilder {
    action: Action,
}

impl ActionBuilder {
    pub fn new() -> ActionBuilder {
        ActionBuilder { 
            action: Action {
                name: String::new(),
                cost: 0.0_f32,
                strategy: Arc::new(Mutex::new(NoopStrategy)),
                prerequisite_beliefs: vec![],
                prohibited_beliefs: vec![],
                completion_beliefs: vec![],
            }
        }
    }

    pub fn build(self) -> Action {
        self.action
    }

    pub fn empty() -> Action {
        ActionBuilder::new().build()
    }

    pub fn name(mut self, name: impl Into<String>) -> ActionBuilder {
        self.action.name = name.into();
        self
    }

    pub fn cost(mut self, cost: f32) -> ActionBuilder {
        self.action.cost = cost;
        self
    }

    pub fn strategy(mut self, strategy: StrategyT) -> ActionBuilder {
        self.action.strategy = strategy;
        self
    }

    pub fn prerequisites(
        mut self,
        beliefs: impl IntoIterator<Item = (impl BeliefSatisfiabilityTest + 'static)>,
    ) -> ActionBuilder {
        self.action.prerequisite_beliefs = beliefs
            .into_iter()
            .map(|test| BeliefTest::new(test))
            .collect();
        self
    }

    pub fn prohibited(
        mut self,
        beliefs: impl IntoIterator<Item = (impl BeliefSatisfiabilityTest + 'static)>,
    ) -> ActionBuilder {
        self.action.prohibited_beliefs = beliefs
            .into_iter()
            .map(|test| BeliefTest::new(test))
            .collect();
        self
    }

    pub fn completion(mut self, beliefs: Vec<Belief>) -> ActionBuilder {
        self.action.completion_beliefs = beliefs;
        self
    }
}

#[cfg(test)]
mod tests {
    mod fn_can_perform {
        use dys_satisfiable::SatisfiableField;
        use crate::ai::action::ActionBuilder;
        use crate::ai::belief::{Belief, BeliefSet, SatisfiableBelief};
    
        #[test]
        fn no_prereqs_no_prohibited_no_beliefs_allowed() {
            let action = ActionBuilder::empty();
            let result = action.can_perform(&BeliefSet::empty());
            assert!(result);
        }
    
        #[test]
        fn no_prereqs_no_prohibited_some_beliefs_allowed() {
            let action = ActionBuilder::empty();
            let result = action.can_perform(&BeliefSet::from(&[
                Belief::HeldBall { ball_id: 1, combatant_id: 1 },
                Belief::OnPlate { plate_id: 1, combatant_id: 1 },
            ]));
            assert!(result);
        }
    
        #[test]
        fn no_prereqs_some_prohibited_no_beliefs_allowed() {
            let action = ActionBuilder::new()
                .prohibited(vec![
                    SatisfiableBelief::OnPlate()
                ])
                .build();
            let result = action.can_perform(&BeliefSet::empty());
            assert!(result);
        }

        #[test]
        fn no_prereqs_some_prohibited_different_belief_types_allowed() {
            let action = ActionBuilder::new()
                .prohibited(vec![
                    SatisfiableBelief::HeldBall()
                ])
                .build();

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id: 1 },
            ]);

            let result = action.can_perform(&belief_set);
            assert!(result);
        }

        #[test]
        fn no_prereqs_some_prohibited_some_beliefs_none_matching_allowed() {
            let prohibited_id = 1;
            let combatant_id = 2;

            let action = ActionBuilder::new()
                .prohibited(vec![
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(prohibited_id))
                ])
                .build();

            let belief_set = BeliefSet::from(&[
                Belief::HeldBall { ball_id: 1, combatant_id },
            ]);

            let result = action.can_perform(&belief_set);
            assert!(result);
        }
    
        #[test]
        fn no_prereqs_some_prohibited_belief_matches_disallowed() {
            let prohibited_id = 1;

            let action = ActionBuilder::new()
                .prohibited(vec![
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(prohibited_id))
                ])
                .build();

            let belief_set = BeliefSet::from(&[
                Belief::HeldBall { ball_id: 1, combatant_id: prohibited_id },
            ]);

            let result = action.can_perform(&belief_set);
            assert!(!result);
        }
    
        #[test]
        fn some_prereqs_no_prohibited_no_beliefs_disallowed() {
            let action = ActionBuilder::new()
                .prerequisites(vec![
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(1))
                ])
                .build();
            let result = action.can_perform(&BeliefSet::empty());
            assert!(!result);
        }

        #[test]
        fn some_prereqs_no_prohibited_different_types_disallowed() {
            let action = ActionBuilder::new()
                .prerequisites(vec![
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(1))
                ])
                .build();

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id: 1 },
            ]);

            let result = action.can_perform(&belief_set);
            assert!(!result);
        }

        #[test]
        fn prereqs_mismatching_disallowed() {
            let prereq_id = 1;
            let combatant_id = 2;
            let action = ActionBuilder::new()
                .prerequisites(vec![
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(prereq_id))
                ])
                .build();

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id },
            ]);

            let result = action.can_perform(&belief_set);
            assert!(!result);
        }
    
        #[test]
        fn some_prereqs_no_prohibited_some_beliefs_some_matching_allowed() {
            let combatant_id = 1;
            let action = ActionBuilder::new()
                .prerequisites(vec![
                    SatisfiableBelief::OnPlate()
                        .combatant_id(SatisfiableField::Exactly(combatant_id))
                ])
                .build();

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id },
            ]);

            let result = action.can_perform(&belief_set);
            assert!(result);
        }
    }
}
