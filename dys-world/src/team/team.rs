use std::sync::{Arc, Mutex};

use crate::combatant::combatant::Combatant;

pub struct Team {
    pub id: u64,
    pub name: String,
    pub combatants: Vec<Arc<Mutex<Combatant>>>
}
