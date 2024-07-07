use serde::{Deserialize, Serialize};

use crate::game_tick::{GameTick, TickPerformance};

#[derive(Debug, Serialize, Deserialize)]
pub struct GameLog {
    ticks: Vec<GameTick>,
    performance: TickPerformance,
}

impl GameLog {
    pub fn from_ticks(ticks: Vec<GameTick>) -> GameLog {
        let perf = ticks
            .iter()
            .map(|game_tick| game_tick.tick_performance())
            .fold(TickPerformance::default(), |acc_perf, next_perf| acc_perf + next_perf.to_owned());

        GameLog {
            ticks,
            performance: perf,
        }
    }

    pub fn ticks(&self) -> &Vec<GameTick> {
        &self.ticks
    }

    pub fn perf_string(&self) -> String {
        self.performance.perf_string()
    }
}