use dys_world::schedule::schedule_game::ScheduleGame;

use crate::{game, game_tick::GameTick};

pub struct GameLog {
    pub schedule_game: ScheduleGame,
    pub ticks: Vec<GameTick>,
}

impl GameLog {
    pub fn perf_string(&self) -> String {
        let (
            total_duration_micros,
            physics_duration_micros,
            balls_duration_micros,
            combatant_duration_micros,
            scoring_duration_micros
        ) = self
            .ticks
            .iter()
            .map(|game_tick| {
                (
                    game_tick.tick_duration.as_micros(),
                    game_tick.physics_duration.as_micros(),
                    game_tick.balls_duration.as_micros(),
                    game_tick.combatant_duration.as_micros(),
                    game_tick.scoring_duration.as_micros()
                )
            })
            .reduce(|(acc_total, acc_phys, acc_balls, acc_com, acc_scoring), (next_total, next_phys, next_balls, next_com, next_scoring)| {
                (
                    acc_total + next_total,
                    acc_phys + next_phys,
                    acc_balls + next_balls,
                    acc_com + next_com,
                    acc_scoring + next_scoring,
                )
            })
            .expect("failed to collect perf statistics from game log");

        format!("{} total μs ({}μs phys, {}μs balls, {}μs combatants, {}μs scoring)",
            total_duration_micros,
            physics_duration_micros,
            balls_duration_micros,
            combatant_duration_micros,
            scoring_duration_micros)
    }
}