use crate::history::recordable::{RecordType, Recordable};

use super::{combatant::Combatant, limb::Limb};

const RECORD_PREFIX: &'static str = "COMB";

pub struct CombatantRecord {  
    pub id: u64,
    pub name: String,
    pub limbs: Vec<Limb>
}

impl RecordType for CombatantRecord {
    fn id(&self) -> String {
        format!("{}-{}", RECORD_PREFIX, self.id)
    }
}

impl Recordable<CombatantRecord> for Combatant {
    fn to_record(&self) -> CombatantRecord {
        CombatantRecord {
            id: self.id,
            name: self.name.clone(),
            limbs: self.limbs.clone()
        }
    }
}