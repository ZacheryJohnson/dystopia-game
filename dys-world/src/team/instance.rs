use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::{
    combatant::instance::CombatantInstance,
    serde::{deserialize_combatants_from_ids, serialize_combatants_to_ids},
};

pub type TeamInstanceId = u64;

#[derive(Deserialize, Serialize)]
pub struct TeamInstance {
    pub id: TeamInstanceId,
    pub name: String,

    #[serde(deserialize_with = "deserialize_combatants_from_ids")]
    #[serde(serialize_with = "serialize_combatants_to_ids")]
    #[serde(rename = "combatant_ids")]
    pub combatants: Vec<Arc<Mutex<CombatantInstance>>>
}