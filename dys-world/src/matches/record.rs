use serde::Serialize;
use crate::history::recordable::{RecordType, Recordable};
use crate::matches::instance::{MatchInstanceId, MatchInstance};
use crate::team::record::TeamRecord;

#[derive(Serialize)]
pub struct MatchRecord {
    match_id: MatchInstanceId,
    away_team: TeamRecord,
    home_team: TeamRecord,
    away_team_score: u32,
    home_team_score: u32,
}

impl RecordType for MatchRecord {
    const RECORD_PREFIX: &'static str = "MATCH";
    type InstanceIdType = MatchInstanceId;

    fn instance_id(&self) -> Self::InstanceIdType {
        self.match_id
    }
}

impl Recordable<MatchRecord> for MatchInstance {
    fn to_record(&self) -> MatchRecord {
        let away_team = self.away_team.lock().unwrap().to_record();
        let home_team = self.home_team.lock().unwrap().to_record();

        MatchRecord {
            match_id: self.match_id,
            away_team,
            home_team,
            away_team_score: 0, // ZJ-TODO: this should instead be impl'ed on GameLog? something else?
            home_team_score: 0, // ZJ-TODO: this should instead be impl'ed on GameLog? something else?
        }
    }
}