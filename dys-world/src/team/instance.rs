use std::sync::{Arc, Mutex};

use crate::combatant::instance::CombatantInstance;

pub type TeamInstanceId = u64;

pub struct TeamInstance {
    pub id: TeamInstanceId,
    pub name: String,
    pub combatants: Vec<Arc<Mutex<CombatantInstance>>>
}
