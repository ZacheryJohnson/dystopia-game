use std::sync::{Arc, Mutex};
use serde::Serializer;
use crate::team::instance::TeamInstance;

pub(crate) fn serialize_team_instance_to_id<S>(
    team: &Arc<Mutex<TeamInstance>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u64(team.lock().unwrap().id)
}