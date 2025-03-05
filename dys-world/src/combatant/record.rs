use serde::Serialize;
use crate::combatant::instance::CombatantInstanceId;
use crate::history::recordable::{RecordType, Recordable};

use super::{instance::CombatantInstance, limb::Limb};

#[derive(Serialize)]
pub struct CombatantRecord {  
    pub combatant_id: CombatantInstanceId,
    pub name: String,
    pub limbs: Vec<Limb>,
}

impl RecordType for CombatantRecord {
    const RECORD_PREFIX: &'static str = "COM";
    type InstanceIdType = CombatantInstanceId;

    fn instance_id(&self) -> Self::InstanceIdType {
        self.combatant_id
    }
}

impl Recordable<CombatantRecord> for CombatantInstance {
    fn to_record(&self) -> CombatantRecord {
        CombatantRecord {
            combatant_id: self.id,
            name: self.name.clone(),
            limbs: self.limbs.clone(),
        }
    }
}