use std::sync::{Arc, Mutex};

use dys_world::combatant::combatant::Combatant;
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

use crate::{game_objects::combatant::{CombatantId, CombatantObject, CombatantState, TeamAlignment}, game_state::GameState, simulation::simulation_event::SimulationEvent};

use super::{agent::Agent, belief::Belief};


pub struct TestAgent {
    combatant: CombatantObject,
    beliefs: Vec<Belief>,
}

#[derive(Default)]
pub struct TestAgentSettings {
    /// If set, `combatant_override` and `combatant_state_override` will be ignored
    combatant_object_override: Option<CombatantObject>,

    combatant_object_id_override: Option<CombatantId>,
    combatant_override: Option<Combatant>,
    combatant_state_override: Option<CombatantState>,
    team_override: Option<TeamAlignment>,
}

impl TestAgent {
    pub fn new() -> TestAgent {
        TestAgent::new_with_settings(TestAgentSettings::default())
    }

    pub fn new_with_settings(settings: TestAgentSettings) -> TestAgent {
        TestAgent {
            combatant: CombatantObject {
                id: settings.combatant_object_id_override.unwrap_or(1),
                combatant: Arc::new(Mutex::new(
                    settings.combatant_override.unwrap_or(Combatant {
                        id: 1,
                        name: String::from("TestCombatant"),
                        limbs: vec![],
                    })
                )),
                combatant_state: settings.combatant_state_override.unwrap_or_default(),
                team: settings.team_override.unwrap_or(TeamAlignment::Home),
                rigid_body_handle: RigidBodyHandle::invalid(),
                collider_handle: ColliderHandle::invalid(),
            },
            beliefs: vec![]
        }
    }

    pub fn set_beliefs(&mut self, beliefs: Vec<Belief>) {
        self.beliefs = beliefs;
    }
}

impl Agent for TestAgent {
    fn combatant(&self) -> &crate::game_objects::combatant::CombatantObject {
        &self.combatant
    }

    fn combatant_mut(&mut self) -> &mut CombatantObject {
        &mut self.combatant
    }
    
    fn beliefs(&self) -> &Vec<super::belief::Belief> {
        &self.beliefs
    }

    fn tick(&mut self, _: Arc<Mutex<GameState>>) -> Vec<SimulationEvent> {
        vec![]
    }
}