use crate::{game_objects::combatant::CombatantObject, game_state::GameState};

use super::{action::Action, belief::Belief, planner::Planner};

pub struct Agent {
    beliefs: Vec<Belief>,

    plan: Vec<Action>,

    current_action: Option<Action>,
}

impl Agent {
    pub fn new() -> Agent {
        Agent {
            beliefs: vec![],
            plan: vec![],
            current_action: None,
        }
    }

    pub fn beliefs(&self) -> &Vec<Belief> {
        &self.beliefs
    }

    pub fn tick(&mut self, combatant: &mut CombatantObject, game_state: &mut GameState) {
        if self.plan.is_empty() {
            self.plan = Planner::plan(&self, combatant, game_state);
            assert!(!self.plan.is_empty(), "Failed to get a valid plan from planner");
        }

        // ZJ-TODO: check for "interrupts" and set a new plan if we received one

        if self.current_action.is_none() {
            self.current_action = Some(self.plan.pop().unwrap());
            assert!(self.current_action.is_some(), "Failed to get a valid action from plan");
        }
        
        let action = self.current_action.as_mut().unwrap();
        action.tick(combatant, game_state);

        if action.is_complete() {
            tracing::debug!("Completed action {}", action.name());
            self.current_action = None;
        }
    }
}