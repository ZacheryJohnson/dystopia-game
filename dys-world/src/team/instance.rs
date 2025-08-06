use std::sync::{Arc, Mutex};
use serde::Serialize;
use ts_rs::TS;
use crate::{
    combatant::instance::CombatantInstance,
    serde::serialize_combatants_to_ids,
};

pub type TeamInstanceId = u32;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct TeamInstance {
    pub id: TeamInstanceId,
    pub name: String,

    #[serde(serialize_with = "serialize_combatants_to_ids")]
    pub combatants: Vec<Arc<Mutex<CombatantInstance>>>
}