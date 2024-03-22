use crate::combatant::limb::Limb;

pub type CombatantId = u64;

pub struct Combatant {
    pub id: CombatantId,
    pub name: String,
    pub limbs: Vec<Limb>
}