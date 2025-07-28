use std::sync::{Arc, Mutex};
use serde::Serialize;
use ts_rs::TS;
use crate::games::serde::serialize_team_instance_to_id;
use crate::team::instance::TeamInstance;
use crate::schedule::calendar::Date;

pub type GameInstanceId = u64;

/// GameInstances are games that are scheduled between two teams.
/// The games may or may not have already been simulated.
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct GameInstance {
    pub game_id: GameInstanceId,
    #[serde(serialize_with = "serialize_team_instance_to_id")]
    pub away_team: Arc<Mutex<TeamInstance>>,
    #[serde(serialize_with = "serialize_team_instance_to_id")]
    pub home_team: Arc<Mutex<TeamInstance>>,
    // #[serde(serialize_with = "serialize_arena_to_id")]
    pub arena_id: u32,
    pub date: Date,
}