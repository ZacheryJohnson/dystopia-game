#![cfg(test)]

use std::sync::{Arc, Mutex};
use rand::SeedableRng;
use rand_pcg::Pcg64;
use dys_world::combatant::instance::CombatantInstance;
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};
use dys_world::arena::Arena;
use dys_world::arena::navmesh::{ArenaNavmesh, ArenaNavmeshConfig};
use dys_world::schedule::calendar::{Date, Month};
use dys_world::schedule::schedule_game::ScheduleGame;
use dys_world::team::instance::TeamInstance;
use crate::{game_objects::combatant::{CombatantId, CombatantObject, CombatantState, TeamAlignment}, game_state::GameState, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::BeliefSet;
use crate::game::Game;
use crate::game_state::{BallsMapT, CollidersMapT, CombatantsMapT, PlatesMapT};
use crate::physics_sim::PhysicsSim;
use crate::simulation::config::SimulationConfig;
use super::{agent::Agent, belief::Belief};

pub struct TestAgent {
    combatant: CombatantObject,
    beliefs: Vec<Belief>,
}

#[derive(Default)]
pub struct TestAgentSettings {
    combatant_object_id_override: Option<CombatantId>,
    combatant_override: Option<CombatantInstance>,
    combatant_state_override: Option<CombatantState>,
    team_override: Option<TeamAlignment>,
}

impl TestAgent {
    pub fn new() -> TestAgent {
        TestAgent::new_with_settings(TestAgentSettings::default())
    }

    pub fn from_combatant(combatant: CombatantObject) -> TestAgent {
        TestAgent {
            combatant,
            beliefs: Vec::new(),
        }
    }

    pub fn new_with_settings(settings: TestAgentSettings) -> TestAgent {
        TestAgent {
            combatant: CombatantObject {
                id: settings.combatant_object_id_override.unwrap_or(1),
                combatant: Arc::new(Mutex::new(
                    settings.combatant_override.unwrap_or(CombatantInstance {
                        id: 1,
                        name: String::from("TestCombatant"),
                        limbs: vec![],
                    })
                )),
                combatant_state: Arc::new(Mutex::new(settings.combatant_state_override.unwrap_or_default())),
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
    fn combatant(&self) -> &CombatantObject {
        &self.combatant
    }
    
    fn beliefs(&self) -> BeliefSet {
        BeliefSet::from(&self.beliefs)
    }

    fn tick(&mut self, _: Arc<Mutex<GameState>>) -> Vec<SimulationEvent> {
        vec![]
    }
}

pub fn make_test_game_state(with_physics_sim: Option<PhysicsSim>) -> Arc<Mutex<GameState>> {
    let game = Game {
        schedule_game: ScheduleGame {
            away_team: Arc::new(Mutex::new(TeamInstance {
                id: 1,
                name: String::from("TestAwayTeam"),
                combatants: vec![],
            })),
            home_team: Arc::new(Mutex::new(TeamInstance {
                id: 2,
                name: String::from("TestHomeTeam"),
                combatants: vec![],
            })),
            arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())), // ZJ-TODO: don't use arena's default values
            date: Date(Month::Arguscorp, 1, 10000),
        },
    };
    let simulation_config = SimulationConfig::default();
    let arena_navmesh = ArenaNavmesh::new_from(
        game.schedule_game.arena.clone(),
        ArenaNavmeshConfig {
            unit_resolution: 1.0
        }
    );

    Arc::new(Mutex::new(GameState {
        game,
        seed: [0; 32],
        rng: Pcg64::from_seed([0; 32]),
        physics_sim: if let Some(physics_sim) = with_physics_sim {
            physics_sim
        } else {
            PhysicsSim::new(simulation_config.ticks_per_second())
        },
        combatants: CombatantsMapT::new(),
        balls: BallsMapT::new(),
        plates: PlatesMapT::new(),
        active_colliders: CollidersMapT::new(),
        home_points: 0,
        away_points: 0,
        current_tick: 0,
        simulation_config,
        arena_navmesh,
    }))
}