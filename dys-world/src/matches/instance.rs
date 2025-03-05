use std::sync::{Arc, Mutex};
use crate::{arena::Arena, team::instance::TeamInstance};
use crate::schedule::calendar::Date;

pub type MatchInstanceId = u64;

/// MatchInstances are matches that are scheduled between two teams.
/// The matches may or may not have already been simulated.
#[derive(Clone)]
pub struct MatchInstance {
    pub match_id: MatchInstanceId,
    pub away_team: Arc<Mutex<TeamInstance>>,
    pub home_team: Arc<Mutex<TeamInstance>>,
    pub arena: Arc<Mutex<Arena>>,
    pub date: Date,
}