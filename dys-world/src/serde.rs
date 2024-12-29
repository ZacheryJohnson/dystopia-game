use std::sync::{Arc, Mutex};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serializer};
use crate::combatant::instance::CombatantInstance;
use crate::team::instance::TeamInstance;

pub(crate) fn serialize_combatants<S>(
    combatants: &Vec<Arc<Mutex<CombatantInstance>>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(combatants.len()))?;
    for combatant in combatants {
        seq.serialize_element(&*combatant.lock().unwrap())?;
    }
    seq.end()
}

pub(crate) fn deserialize_combatants<'de, D>(
    deserializer: D,
) -> Result<Vec<Arc<Mutex<CombatantInstance>>>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<Arc<Mutex<CombatantInstance>>>::deserialize(deserializer)
}

pub(crate) fn serialize_combatants_to_ids<S>(
    combatants: &Vec<Arc<Mutex<CombatantInstance>>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(combatants.len()))?;
    for combatant in combatants {
        seq.serialize_element(&combatant.lock().unwrap().id)?;
    }
    seq.end()
}

pub(crate) fn deserialize_combatants_from_ids<'de, D>(
    _: D,
) -> Result<Vec<Arc<Mutex<CombatantInstance>>>, D::Error>
where
    D: Deserializer<'de>,
{
    unimplemented!("ZJ-TODO")
}

pub(crate) fn serialize_teams<S>(
    teams: &Vec<Arc<Mutex<TeamInstance>>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(teams.len()))?;
    for team in teams {
        seq.serialize_element(&*team.lock().unwrap())?;
    }
    seq.end()
}

pub(crate) fn deserialize_teams<'de, D>(
    deserializer: D,
) -> Result<Vec<Arc<Mutex<TeamInstance>>>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<Arc<Mutex<TeamInstance>>>::deserialize(deserializer)
}