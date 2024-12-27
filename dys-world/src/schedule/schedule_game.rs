use std::sync::{Arc, Mutex};

use crate::{arena::Arena, team::instance::TeamInstance};

use super::calendar::Date;

#[derive(Clone)]
pub struct ScheduleGame {
    pub away_team: Arc<Mutex<TeamInstance>>,
    pub home_team: Arc<Mutex<TeamInstance>>,
    pub arena: Arc<Mutex<Arena>>,
    pub date: Date,
}