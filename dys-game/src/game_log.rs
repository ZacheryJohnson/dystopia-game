use dys_world::schedule::schedule_game::ScheduleGame;

use crate::game_tick::{GameTick, TickPerformance};

pub struct GameLog {
    pub schedule_game: ScheduleGame,
    pub ticks: Vec<GameTick>,
}

impl GameLog {
    pub fn perf_string(&self) -> String {
        let perf = self
            .ticks
            .iter()
            .map(|game_tick| game_tick.tick_performance())
            .fold(TickPerformance::default(), |acc_perf, next_perf| acc_perf + next_perf.to_owned());

        perf.perf_string()
    }
}