use std::sync::{Arc, Mutex};
use serde::Serialize;
use crate::{
    combatant::instance::CombatantInstance,
    serde::serialize_combatants_to_ids,
};

pub type TeamInstanceId = u64;

#[derive(Debug, Serialize)]
pub struct TeamInstance {
    pub id: TeamInstanceId,
    pub name: String,

    #[serde(serialize_with = "serialize_combatants_to_ids")]
    pub combatants: Vec<Arc<Mutex<CombatantInstance>>>
}