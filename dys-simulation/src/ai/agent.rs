use std::sync::{Arc, Mutex};
use crate::{game_objects::combatant::CombatantObject, game_state::GameState};
use crate::simulation::simulation_event::PendingSimulationEvent;
use super::belief::BeliefSet;

pub trait Agent {
    fn combatant(&self) -> &CombatantObject;

    fn beliefs(&self) -> BeliefSet;

    fn tick(&mut self, game_state: Arc<Mutex<GameState>>) -> Vec<PendingSimulationEvent>;
}
