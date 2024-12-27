use std::sync::{Arc, Mutex};

use crate::{combatant::definition::CombatantDefinition, team::definition::TeamDefinition};

#[derive(Clone)]
pub struct World {
    pub combatants: Vec<Arc<Mutex<CombatantDefinition>>>,
    pub teams: Vec<Arc<Mutex<TeamDefinition>>>,
}