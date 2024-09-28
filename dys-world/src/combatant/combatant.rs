use crate::combatant::limb::Limb;

pub type CombatantId = u64;

#[derive(Debug)]
pub struct Combatant {
    pub id: CombatantId,
    pub name: String,
    pub limbs: Vec<Limb>
}

impl Combatant {
    /// ZJ-TODO: HACK: calculate this from combatant limbs and modifiers
    pub fn move_speed(&self) -> f32 {
        2.0_f32
    }
}