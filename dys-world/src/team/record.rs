use serde::Serialize;
use crate::{combatant::record::CombatantRecord, history::recordable::{RecordType, Recordable}};

use super::instance::{TeamInstance, TeamInstanceId};

#[derive(Serialize)]
pub struct TeamRecord {
    pub team_id: TeamInstanceId,
    pub combatant_records: Vec<CombatantRecord>,
}

impl RecordType for TeamRecord {
    const RECORD_PREFIX: &'static str = "TEAM";
    type InstanceIdType = TeamInstanceId;

    fn instance_id(&self) -> Self::InstanceIdType {
        self.team_id
    }
}

impl Recordable<TeamRecord> for TeamInstance {
    fn to_record(&self) -> TeamRecord {
        let mut combatant_records = vec![];
        for combatant_arc in &self.combatants {
            let combatant_instance = combatant_arc.lock().unwrap();
            combatant_records.push(combatant_instance.to_record());
        }

        TeamRecord {
            team_id: self.id,
            combatant_records,
        }
    }
}