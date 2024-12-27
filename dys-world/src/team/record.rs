use crate::{combatant::record::CombatantRecord, history::recordable::{RecordType, Recordable}};

use super::instance::TeamInstance;

const RECORD_PREFIX: &str = "TEAM";

pub struct TeamRecord {
    pub id: u64,
    pub combatant_records: Vec<CombatantRecord>
}

impl RecordType for TeamRecord {
    fn id(&self) -> String {
        format!("{}-{}", RECORD_PREFIX, self.id)
    }
}

impl Recordable<TeamRecord> for TeamInstance {
    fn to_record(&self) -> TeamRecord {
        todo!()
    }
}