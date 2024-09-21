use crate::{game_objects::combatant::CombatantObject, game_state::GameState};

use super::belief::Belief;

pub trait Agent {
    fn combatant(&self) -> &CombatantObject;

    fn combatant_mut(&mut self) -> &mut CombatantObject;

    fn beliefs(&self) -> &Vec<Belief>;

    fn tick(&mut self, game_state: &mut GameState);
}