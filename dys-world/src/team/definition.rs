use std::sync::{Arc, Mutex};

use crate::combatant::definition::CombatantDefinition;

pub struct TeamDefinition {
    pub id: u64,
    pub name: String,
    pub combatants: Vec<Arc<Mutex<CombatantDefinition>>>
}
