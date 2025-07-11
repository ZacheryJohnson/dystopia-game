use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use dys_satisfiable::SatisfiabilityTest;
use crate::ai::beliefs::belief_set::BeliefSet;
use crate::game_state::GameState;
use crate::simulation::simulation_event::PendingSimulationEvent;

use super::agent::Agent;
use super::belief::{Belief, BeliefSatisfiabilityTest, BeliefTest};
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

    /// Promised beliefs are concrete beliefs that will be applied by sensors
    /// rather than the action itself.
    promised_beliefs: Vec<Belief>,

    /// Concrete beliefs applied once the action completes successfully.
    completion_beliefs: Vec<Belief>,

    /// Concrete beliefs that are applied to OTHERS once the action completes successfully.
    broadcast_on_completion_beliefs: Vec<Belief>,

    /// Beliefs that will be consumed upon completing the action
    consumed_beliefs: Vec<BeliefTest>,
}

impl Action {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn cost(&self) -> f32 {
        self.cost
    }

    /// Given an agent's belief set, can this action currently be performed?
    #[tracing::instrument(name = "action::can_perform", skip_all, level = "trace")]
    pub fn can_perform(&self, owned_beliefs: &BeliefSet) -> bool {
        let all_prereqs = self
            .prerequisite_beliefs
            .iter()
            .all(|belief| owned_beliefs.can_satisfy(belief));

        let none_prohibited = self
            .prohibited_beliefs
            .iter()
            .inspect(|belief| tracing::debug!("Can satisfy prohibited {:?} = {}", belief, owned_beliefs.can_satisfy(*belief)))
            .all(|belief| !owned_beliefs.can_satisfy(belief));

        all_prereqs && none_prohibited
    }

    pub fn should_interrupt(&self, owned_beliefs: &BeliefSet) -> bool {
        self.strategy.lock().unwrap().should_interrupt(owned_beliefs)
    }

    /// If performed, could this action satisfy the given belief test?
    #[tracing::instrument(name = "action::can_perform", skip_all, level = "trace")]
    pub fn can_satisfy(&self, belief_test: BeliefTest) -> bool {
        self
            .completion_beliefs
            .iter()
            .chain(self.promised_beliefs.iter())
            .any(|belief| belief_test.satisfied_by(*belief))
    }

    pub fn is_complete(&self, agent_beliefs: &BeliefSet) -> bool {
        let strategy_is_complete = self.strategy.lock().unwrap().is_complete();
        let all_promised_beliefs_satisfied = if self.promised_beliefs.is_empty() {
            false
        } else {
            self
                .promised_beliefs
                .iter()
                .all(|promised_belief| agent_beliefs.can_satisfy(promised_belief))
        };

        strategy_is_complete || all_promised_beliefs_satisfied
    }

    #[tracing::instrument(name = "action::tick", skip_all, level = "trace")]
    pub fn tick(
        &mut self,
        agent: &impl Agent,
        game_state: Arc<Mutex<GameState>>,
    ) -> Option<Vec<PendingSimulationEvent>> {
        if !self.can_perform(&agent.beliefs()) {
            tracing::debug!("Cannot perform action (action fail) {}", self.name());
            return None;
        }

        let mut strategy = self.strategy.lock().unwrap();
        if !strategy.can_perform(&agent.beliefs()) {
            tracing::debug!("Cannot perform action (strategy fail) {}", self.name());
            return None;
        }

        tracing::trace!("Executing strategy {}", strategy.name());

        strategy.tick(agent, game_state)
    }

    pub fn completion_beliefs(&self) -> &Vec<Belief> {
        &self.completion_beliefs
    }

    pub fn broadcast_beliefs(&self) -> &Vec<Belief> {
        &self.broadcast_on_completion_beliefs
    }

    pub fn prohibited_beliefs(&self) -> &Vec<BeliefTest> {
        &self.prohibited_beliefs
    }

    pub fn promised_beliefs(&self) -> &Vec<Belief> {
        &self.promised_beliefs
    }

    pub fn prerequisite_beliefs(&self) -> &Vec<BeliefTest> {
        &self.prerequisite_beliefs
    }

    pub fn consumed_beliefs(&self) -> &Vec<BeliefTest> { &self.consumed_beliefs }
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
            .field("broadcast_beliefs", &self.broadcast_on_completion_beliefs)
            .field("promised_beliefs", &self.promised_beliefs)
            .field("consumed_beliefs", &self.consumed_beliefs)
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
                broadcast_on_completion_beliefs: vec![],
                promised_beliefs: vec![],
                consumed_beliefs: vec![],
            }
        }
    }

    #[cfg(test)]
    pub fn empty() -> Action {
        ActionBuilder::new().build()
    }

    pub fn build(self) -> Action {
        self.action
    }

    pub fn name(mut self, name: impl Into<String>) -> ActionBuilder {
        self.action.name = name.into();
        self
    }

    pub fn cost(mut self, cost: f32) -> ActionBuilder {
        self.action.cost = cost;
        self
    }

    pub fn strategy(mut self, strategy: impl Strategy + 'static) -> ActionBuilder {
        self.action.strategy = Arc::new(Mutex::new(strategy));
        self
    }

    pub fn requires(
        mut self,
        belief: impl BeliefSatisfiabilityTest + 'static,
    ) -> ActionBuilder {
        self.action.prerequisite_beliefs.push(BeliefTest::new(belief));
        self
    }

    pub fn prohibits(
        mut self,
        belief: impl BeliefSatisfiabilityTest + 'static,
    ) -> ActionBuilder {
        self.action.prohibited_beliefs.push(BeliefTest::new(belief));
        self
    }

    pub fn completion(mut self, beliefs: Vec<Belief>) -> ActionBuilder {
        self.action.completion_beliefs = beliefs;
        self
    }

    pub fn broadcasts(mut self, beliefs: Vec<Belief>) -> ActionBuilder {
        self.action.broadcast_on_completion_beliefs = beliefs;
        self
    }

    pub fn promises(mut self, belief: Belief) -> Self {
        self.action.promised_beliefs.push(belief);
        self
    }

    pub fn consumes(mut self, belief: impl BeliefSatisfiabilityTest + 'static) -> Self {
        self.action.consumed_beliefs.push(BeliefTest::new(belief));
        self
    }
}

impl From<ActionBuilder> for Action {
    fn from(value: ActionBuilder) -> Self {
        value.action
    }
}

#[cfg(test)]
mod tests {
    use dys_satisfiable::SatisfiableField;
    use crate::ai::action::ActionBuilder;
    use crate::ai::belief::{Belief, SatisfiableBelief};

    mod fn_can_perform {
        use crate::ai::beliefs::belief_set::BeliefSet;
        use super::*;

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
                .prohibits(SatisfiableBelief::OnPlate())
                .build();
            let result = action.can_perform(&BeliefSet::empty());
            assert!(result);
        }

        #[test]
        fn no_prereqs_some_prohibited_different_belief_types_allowed() {
            let action = ActionBuilder::new()
                .prohibits(SatisfiableBelief::HeldBall())
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
                .prohibits(
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(prohibited_id))
                )
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
                .prohibits(
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(prohibited_id))
                )
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
                .requires(
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(1))
                )
                .build();
            let result = action.can_perform(&BeliefSet::empty());
            assert!(!result);
        }

        #[test]
        fn some_prereqs_no_prohibited_different_types_disallowed() {
            let action = ActionBuilder::new()
                .requires(
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(1))
                )
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
                .requires(
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(prereq_id))
                )
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
                .requires(
                    SatisfiableBelief::OnPlate()
                        .combatant_id(SatisfiableField::Exactly(combatant_id))
                )
                .build();

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id },
            ]);

            let result = action.can_perform(&belief_set);
            assert!(result);
        }

        #[test]
        fn test() {
            let combatant_id = 1;
            let action = ActionBuilder::new()
                .requires(
                    SatisfiableBelief::OnPlate()
                        .combatant_id(SatisfiableField::Exactly(combatant_id))
                )
                .prohibits(
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(combatant_id))
                )
                .prohibits(
                    SatisfiableBelief::CombatantIsStunned()
                        .combatant_id(SatisfiableField::Exactly(combatant_id))
                )
                .build();

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id },
            ]);

            assert!(action.can_perform(&belief_set));

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id },
                Belief::CombatantIsStunned { combatant_id },
            ]);

            assert!(!action.can_perform(&belief_set));

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id },
                Belief::HeldBall { ball_id: 1, combatant_id },
            ]);

            assert!(!action.can_perform(&belief_set));

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id },
                Belief::CombatantIsStunned { combatant_id },
                Belief::HeldBall { ball_id: 1, combatant_id },
            ]);

            assert!(!action.can_perform(&belief_set));

            let belief_set = BeliefSet::from(&[
                Belief::OnPlate { plate_id: 1, combatant_id },
                Belief::CombatantIsStunned { combatant_id },
                Belief::HeldBall { ball_id: 1, combatant_id },
            ]);

            assert!(!action.can_perform(&belief_set));
        }
    }

    mod fn_can_satisfy {
        use super::*;

        #[test]
        fn completion_belief_satisfies() {
            let combatant_id = 1;
            let ball_id = 1;
            let action = ActionBuilder::new()
                .completion(vec![Belief::HeldBall {combatant_id, ball_id}])
                .build();

            assert!(action.can_satisfy(SatisfiableBelief::HeldBall()
                .into()));

            assert!(action.can_satisfy(SatisfiableBelief::HeldBall()
                .combatant_id(SatisfiableField::Exactly(combatant_id))
                .into()));

            assert!(action.can_satisfy(SatisfiableBelief::HeldBall()
                .ball_id(SatisfiableField::Exactly(ball_id))
                .into()));

            assert!(action.can_satisfy(SatisfiableBelief::HeldBall()
                .combatant_id(SatisfiableField::Exactly(combatant_id))
                .ball_id(SatisfiableField::Exactly(ball_id))
                .into()));
        }

        #[test]
        fn promised_belief_satisfies() {
            let combatant_id = 1;
            let ball_id = 1;
            let action = ActionBuilder::new()
                .promises(Belief::HeldBall { combatant_id, ball_id })
                .build();

            assert!(action.can_satisfy(SatisfiableBelief::HeldBall()
                .into()));

            assert!(action.can_satisfy(SatisfiableBelief::HeldBall()
                .combatant_id(SatisfiableField::Exactly(combatant_id))
                .into()));

            assert!(action.can_satisfy(SatisfiableBelief::HeldBall()
                .ball_id(SatisfiableField::Exactly(ball_id))
                .into()));

            assert!(action.can_satisfy(SatisfiableBelief::HeldBall()
                .combatant_id(SatisfiableField::Exactly(combatant_id))
                .ball_id(SatisfiableField::Exactly(ball_id))
                .into()));
        }
    }

    mod fn_is_complete {
        use std::sync::{Arc, Mutex};
        use crate::ai::agent::Agent;
        use crate::ai::beliefs::belief_set::BeliefSet;
        use crate::ai::strategy::Strategy;
        use crate::game_state::GameState;
        use crate::simulation::simulation_event::PendingSimulationEvent;
        use super::*;

        struct StrategyAlwaysIsComplete;
        impl Strategy for StrategyAlwaysIsComplete {
            fn name(&self) -> String { String::from("StrategyAlwaysIsComplete") }

            fn can_perform(&self, _: &BeliefSet) -> bool { true }

            fn should_interrupt(&self, _: &BeliefSet) -> bool {
                false
            }
            
            fn is_complete(&self) -> bool { true }

            fn tick(&mut self, _: &dyn Agent, _: Arc<Mutex<GameState>>) -> Option<Vec<PendingSimulationEvent>> {
                None
            }
        }

        struct StrategyNeverIsComplete;
        impl Strategy for StrategyNeverIsComplete {
            fn name(&self) -> String { String::from("StrategyNeverIsComplete") }

            fn can_perform(&self, _: &BeliefSet) -> bool { true }

            fn should_interrupt(&self, _: &BeliefSet) -> bool {
                false
            }

            fn is_complete(&self) -> bool { false }

            fn tick(&mut self, _: &dyn Agent, _: Arc<Mutex<GameState>>) -> Option<Vec<PendingSimulationEvent>> {
                None
            }
        }

        #[test]
        fn strategy_is_complete_action_is_complete() {
            let action = ActionBuilder::new()
                .strategy(StrategyAlwaysIsComplete)
                .build();

            let belief_set = BeliefSet::from(&[]);

            assert!(action.is_complete(&belief_set));
        }

        #[test]
        fn strategy_is_not_complete_has_belief_action_is_complete() {
            let plate_id = 1;
            let combatant_id = 1;

            let on_plate_belief = Belief::OnPlate { plate_id, combatant_id };

            let action = ActionBuilder::new()
                .strategy(StrategyNeverIsComplete)
                .promises(on_plate_belief.clone())
                .build();

            let belief_set = BeliefSet::from(&[
                on_plate_belief,
            ]);

            assert!(action.is_complete(&belief_set));
        }

        #[test]
        fn strategy_is_not_complete_missing_belief_action_is_not_complete() {
            let plate_id = 1;
            let combatant_id = 1;

            let action = ActionBuilder::new()
                .strategy(StrategyNeverIsComplete)
                .promises(Belief::OnPlate { plate_id, combatant_id })
                .build();

            let belief_set = BeliefSet::from(&[
                Belief::HeldBall { combatant_id, ball_id: 1 },
                Belief::OnPlate { combatant_id, plate_id: 2},
            ]);

            assert!(!action.is_complete(&belief_set));
        }
    }
}
