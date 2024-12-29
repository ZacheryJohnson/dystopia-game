use serde::{Deserialize, Serialize};
use crate::combatant::limb::Limb;

pub type CombatantInstanceId = u64;

#[derive(Debug, Deserialize, Serialize)]
pub struct CombatantInstance {
    pub id: CombatantInstanceId,
    pub name: String,
    pub limbs: Vec<Limb>
}

impl CombatantInstance {
    /// ZJ-TODO: HACK: calculate this from combatant limbs and modifiers
    pub fn move_speed(&self) -> f32 {
        1.0_f32
    }
}
