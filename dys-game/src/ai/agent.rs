use crate::{game_objects::combatant::CombatantObject, game_state::GameState, simulation::simulation_event::SimulationEvent};

use super::belief::Belief;

pub trait Agent {
    fn combatant(&self) -> &CombatantObject;

    fn beliefs(&self) -> &Vec<Belief>;

    fn tick(&mut self, game_state: &mut GameState) -> Vec<SimulationEvent>;
}