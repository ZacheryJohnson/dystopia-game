use std::sync::{Arc, Mutex};

use crate::{arena::Arena, team::definition::TeamDefinition};

use super::calendar::Date;

#[derive(Clone)]
pub struct ScheduleGame {
    pub away_team: Arc<Mutex<TeamDefinition>>,
    pub home_team: Arc<Mutex<TeamDefinition>>,
    pub arena: Arc<Mutex<Arena>>,
    pub date: Date,
}