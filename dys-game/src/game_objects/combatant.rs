use std::sync::{Arc, Mutex};

use dys_world::combatant::combatant::Combatant;

use crate::game_state::Vector3;

pub type CombatantId = u64;

pub struct CombatantObject {
    // ------
    // Initial state
    // ------

    pub id: CombatantId,
    pub combatant: Arc<Mutex<Combatant>>,

    // ------
    // Active state
    // ------
    
    pub combatant_state: CombatantState,
    pub world_position: Vector3,

}

pub enum CombatantState {
    Idle,
}

impl CombatantObject {
    pub fn new(id: CombatantId, combatant: Arc<Mutex<Combatant>>, position: Vector3) -> CombatantObject {
        CombatantObject {
            id,
            combatant,
            combatant_state: CombatantState::Idle,
            world_position: position
        }
    }
}