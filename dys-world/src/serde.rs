use std::sync::{Arc, Mutex};
use serde::ser::SerializeSeq;
use serde::Serializer;
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