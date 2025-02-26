use super::belief::{BeliefSatisfiabilityTest, BeliefTest};

/// Goals are a set of beliefs about the world.
#[derive(Debug)]
pub struct Goal {
    /// Name of the goal.
    name: String,

    /// Priority of the goal. 
    /// Goals with higher priorities will be selected and executed before goals with lower priorities.
    priority: u32,

    /// Beliefs about the world that we would like to be true.
    /// 
    /// As an example, a combatant might have a "ScorePoints" goal.
    /// This would require the combatant to have a "StandingOnPlate" belief.
    desired_beliefs: Vec<BeliefTest>,

    /// Can this goal be completed multiple times in a row?
    /// By default, false.
    repeatable: bool,
}

impl Goal {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn desired_beliefs(&self) -> Vec<BeliefTest> {
        self.desired_beliefs.clone()
    }

    pub fn repeatable(&self) -> bool {
        self.repeatable
    }
}

pub(super) struct GoalBuilder {
    goal: Goal
}

impl GoalBuilder {
    pub(super) fn new() -> GoalBuilder {
        GoalBuilder {
            goal: Goal {
                name: String::new(),
                priority: 0_u32,
                desired_beliefs: vec![],
                repeatable: false,
            },  
        }
    }

    pub(super) fn build(self) -> Goal {
        self.goal
    }

    pub(super) fn name(mut self, name: impl Into<String>) -> GoalBuilder {
        self.goal.name = name.into();
        self
    }

    pub(super) fn priority(mut self, priority: u32) -> GoalBuilder {
        self.goal.priority = priority;
        self
    }

    pub(super) fn repeatable(mut self, repeatable: bool) -> GoalBuilder {
        self.goal.repeatable = repeatable;
        self
    }

    pub(super) fn desired_belief(mut self, desired_belief: impl BeliefSatisfiabilityTest + 'static) -> GoalBuilder {
        self.goal.desired_beliefs.push(desired_belief.into());
        self
    }

    pub(super) fn desired_beliefs(
        mut self,
        desired_beliefs: impl IntoIterator<Item = (impl BeliefSatisfiabilityTest + 'static)>,
    ) -> GoalBuilder {
        self.goal.desired_beliefs = desired_beliefs
            .into_iter()
            .map(|belief_test| belief_test.into())
            .collect();
        self
    }
}