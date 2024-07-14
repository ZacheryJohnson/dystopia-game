use std::sync::{Arc, Mutex};

use crate::{combatant::combatant::Combatant, team::team::Team};

#[derive(Clone)]
pub struct World {
    pub combatants: Vec<Arc<Mutex<Combatant>>>,
    pub teams: Vec<Arc<Mutex<Team>>>,
}