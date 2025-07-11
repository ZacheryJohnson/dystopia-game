use std::sync::{Arc, Mutex};
use crate::{game_objects::combatant::CombatantObject, game_state::GameState};
use crate::ai::beliefs::belief_set::BeliefSet;
use crate::simulation::simulation_event::PendingSimulationEvent;

pub trait Agent {
    fn combatant(&self) -> &CombatantObject;

    fn beliefs(&self) -> BeliefSet;

    fn tick(&mut self, game_state: Arc<Mutex<GameState>>) -> Vec<PendingSimulationEvent>;
}
