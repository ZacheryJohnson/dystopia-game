use std::sync::{Arc, Mutex};

use dys_world::combatant::combatant::Combatant;
use rapier3d::na::Vector3;

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
    pub world_position: Vector3<f32>,

}

pub enum CombatantState {
    Idle,
}

impl CombatantObject {
    pub fn new(id: CombatantId, combatant: Arc<Mutex<Combatant>>, position: Vector3<f32>) -> CombatantObject {
        CombatantObject {
            id,
            combatant,
            combatant_state: CombatantState::Idle,
            world_position: position
        }
    }
}