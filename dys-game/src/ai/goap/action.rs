use crate::game_objects::combatant::CombatantObject;
use crate::game_state::GameState;

use super::belief::Belief;
use super::strategies::noop::NoopStrategy;
use super::strategy::Strategy;

pub(super) struct Action {
    /// Name of the action
    name: String,

    /// Cost of performing this action
    /// Arbitrary float value
    cost: f32,

    strategy: Box<dyn Strategy>,

    /// Beliefs required for the action to be taken
    prerequisite_beliefs: Vec<Belief>,

    /// Beliefs that prevent the action from being taken
    prohibited_beliefs: Vec<Belief>,

    /// Beliefs applied once the action completes successfully
    completion_beliefs: Vec<Belief>,
}

impl Action {
    pub(super) fn name(&self) -> String {
        self.name.clone()
    }

    pub(super) fn can_perform(&self, owned_beliefs: &[Belief]) -> bool {
        let all_prereqs = self.prerequisite_beliefs.iter().all(|belief| owned_beliefs.contains(belief));
        let none_prohibited = self.prohibited_beliefs.iter().all(|belief| !owned_beliefs.contains(belief));

        all_prereqs && none_prohibited
    }

    pub(super) fn is_complete(&self) -> bool {
        self.strategy.is_complete()
    }

    pub(super) fn tick(&mut self, combatant: &mut CombatantObject, game_state: &mut GameState) {
        if !self.strategy.can_perform() {
            return;
        }

        self.strategy.tick(combatant, game_state);

        if self.strategy.is_complete() {
            todo!("grant completion beliefs")
        }
    }

    pub(super) fn completion_beliefs(&self) -> Vec<Belief> {
        self.completion_beliefs.clone()
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
                strategy: Box::new(NoopStrategy),
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

    pub fn strategy(mut self, strategy: Box<dyn Strategy>) -> ActionBuilder {
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
        use crate::ai::goap::action::ActionBuilder;
        use crate::ai::goap::belief::Belief;
    
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