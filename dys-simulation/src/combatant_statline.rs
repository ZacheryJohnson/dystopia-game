use serde::{Deserialize, Serialize};
use crate::game_objects::combatant::CombatantId;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CombatantStatline {
    pub combatant_id: CombatantId,
    pub points_scored: u8,
    pub balls_thrown: u16,
    pub throws_hit: u16,
    pub combatants_shoved: u16,
}