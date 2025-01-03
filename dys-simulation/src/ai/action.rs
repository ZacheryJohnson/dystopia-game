use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::game_state::GameState;
use crate::simulation::simulation_event::SimulationEvent;

use super::agent::Agent;
use super::belief::Belief;
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

    /// Beliefs required for the action to be taken
    prerequisite_beliefs: Vec<Belief>,

    /// Beliefs that prevent the action from being taken
    prohibited_beliefs: Vec<Belief>,

    /// Beliefs applied once the action completes successfully
    completion_beliefs: Vec<Belief>,
}

impl Action {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn cost(&self) -> f32 {
        self.cost
    }

    pub fn can_perform(&self, owned_beliefs: &[Belief]) -> bool {
        let all_prereqs = self.prerequisite_beliefs.iter().all(|belief| owned_beliefs.contains(belief));
        let none_prohibited = self.prohibited_beliefs.iter().all(|belief| !owned_beliefs.contains(belief));
        let can_perform_strategy = self.strategy.lock().unwrap().can_perform(owned_beliefs);

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

    pub fn prohibited_beliefs(&self) -> &Vec<Belief> {
        &self.prohibited_beliefs
    }

    pub fn prerequisite_beliefs(&self) -> &Vec<Belief> {
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

    pub fn prerequisites(mut self, beliefs: Vec<Belief>) -> ActionBuilder {
        self.action.prerequisite_beliefs = beliefs;
        self
    }

    pub fn prohibited(mut self, beliefs: Vec<Belief>) -> ActionBuilder {
        self.action.prohibited_beliefs = beliefs;
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
        use crate::ai::action::ActionBuilder;
        use crate::ai::belief::Belief;
    
        #[test]
        fn no_prereqs_no_prohibited_no_beliefs_allowed() {
            let action = ActionBuilder::empty();
            let result = action.can_perform(&[]);
            assert!(result);
        }
    
        #[test]
        fn no_prereqs_no_prohibited_some_beliefs_allowed() {
            let action = ActionBuilder::empty();
            let result = action.can_perform(&[Belief::SelfHasBall, Belief::SelfOnPlate]);
            assert!(result);
        }
    
        #[test]
        fn no_prereqs_some_prohibited_no_beliefs_allowed() {
            let action = ActionBuilder::empty();
            let result = action.can_perform(&[]);
            assert!(result);
        }
    
        #[test]
        fn no_prereqs_some_prohibited_some_beliefs_none_matching_allowed() {
            let action = ActionBuilder::new()
                .prohibited(vec![Belief::SelfHasBall])
                .build();
            let result = action.can_perform(&[Belief::SelfOnPlate]);
            assert!(result);
        }
    
        #[test]
        fn no_prereqs_some_prohibited_some_beliefs_some_matching_disallowed() {
            let action = ActionBuilder::new()
                .prohibited(vec![Belief::SelfHasBall])
                .build();
            let result = action.can_perform(&[Belief::SelfHasBall]);
            assert!(!result);
        }
    
        #[test]
        fn some_prereqs_no_prohibited_no_beliefs_disallowed() {
            let action = ActionBuilder::new()
                .prerequisites(vec![Belief::SelfHasBall])
                .build();
            let result = action.can_perform(&[]);
            assert!(!result);
        }
    
        #[test]
        fn some_prereqs_no_prohibited_some_beliefs_none_matching_disallowed() {
            let action = ActionBuilder::new()
                .prerequisites(vec![Belief::SelfHasBall])
                .build();
            let result = action.can_perform(&[Belief::SelfOnPlate]);
            assert!(!result);
        }
    
        #[test]
        fn some_prereqs_no_prohibited_some_beliefs_some_matching_allowed() {
            let action = ActionBuilder::new()
                .prerequisites(vec![Belief::SelfHasBall])
                .build();
            let result = action.can_perform(&[Belief::SelfHasBall]);
            assert!(result);
        }
    }
}
