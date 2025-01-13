use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::{
    combatant::instance::CombatantInstance,
    serde::{deserialize_combatants, deserialize_teams, serialize_combatants, serialize_teams},
    team::instance::TeamInstance,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct World {
    #[serde(deserialize_with = "deserialize_combatants")]
    #[serde(serialize_with = "serialize_combatants")]
    pub combatants: Vec<Arc<Mutex<CombatantInstance>>>,

    #[serde(deserialize_with = "deserialize_teams")]
    #[serde(serialize_with = "serialize_teams")]
    pub teams: Vec<Arc<Mutex<TeamInstance>>>,
}