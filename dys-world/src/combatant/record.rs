use crate::history::recordable::{RecordType, Recordable};

use super::{instance::CombatantInstance, limb::Limb};

const RECORD_PREFIX: &str = "COMB";

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

impl Recordable<CombatantRecord> for CombatantInstance {
    fn to_record(&self) -> CombatantRecord {
        CombatantRecord {
            id: self.id,
            name: self.name.clone(),
            limbs: self.limbs.clone()
        }
    }
}