use std::sync::{Arc, Mutex};
use serde::Serialize;
use crate::matches::serde::serialize_team_instance_to_id;
use crate::{arena::{Arena, serde::serialize_arena_to_id}, team::instance::TeamInstance};
use crate::schedule::calendar::Date;

pub type MatchInstanceId = u64;

/// MatchInstances are matches that are scheduled between two teams.
/// The matches may or may not have already been simulated.
#[derive(Clone, Debug, Serialize)]
pub struct MatchInstance {
    pub match_id: MatchInstanceId,
    #[serde(serialize_with = "serialize_team_instance_to_id")]
    pub away_team: Arc<Mutex<TeamInstance>>,
    #[serde(serialize_with = "serialize_team_instance_to_id")]
    pub home_team: Arc<Mutex<TeamInstance>>,
    // #[serde(serialize_with = "serialize_arena_to_id")]
    pub arena_id: u32,
    pub date: Date,
}