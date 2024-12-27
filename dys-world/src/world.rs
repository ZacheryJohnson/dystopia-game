use std::sync::{Arc, Mutex};

use crate::{combatant::instance::CombatantInstance, team::instance::TeamInstance};

#[derive(Clone)]
pub struct World {
    pub combatants: Vec<Arc<Mutex<CombatantInstance>>>,
    pub teams: Vec<Arc<Mutex<TeamInstance>>>,
}