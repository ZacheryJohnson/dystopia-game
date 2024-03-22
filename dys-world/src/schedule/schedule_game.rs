use std::sync::{Arc, Mutex};

use crate::{arena::Arena, team::team::Team};

use super::calendar::Date;

#[derive(Clone)]
pub struct ScheduleGame {
    pub away_team: Arc<Mutex<Team>>,
    pub home_team: Arc<Mutex<Team>>,
    pub arena: Arc<Mutex<Arena>>,
    pub date: Date,
}