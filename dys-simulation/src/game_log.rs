use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::game_state::GameState;
use crate::game_tick::{GameTick, TickPerformance};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GameLog {
    home_score: u16,
    away_score: u16,
    ticks: Vec<GameTick>,
    performance: TickPerformance,
}

impl GameLog {
    pub fn from_ticks(ticks: Vec<GameTick>, game_state: Arc<Mutex<GameState>>) -> GameLog {
        let perf = ticks
            .iter()
            .map(|game_tick| game_tick.tick_performance())
            .fold(TickPerformance::default(), |acc_perf, next_perf| acc_perf + next_perf.to_owned());

        let game_state = game_state.lock().unwrap();

        GameLog {
            home_score: game_state.home_points,
            away_score: game_state.away_points,
            ticks,
            performance: perf,
        }
    }

    pub fn home_score(&self) -> u16 { self.home_score }

    pub fn away_score(&self) -> u16 { self.away_score }

    pub fn ticks(&self) -> &Vec<GameTick> {
        &self.ticks
    }

    pub fn perf_string(&self) -> String {
        self.performance.perf_string()
    }
}