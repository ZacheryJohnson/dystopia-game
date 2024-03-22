use dys_world::schedule::schedule_game::ScheduleGame;

use crate::game_tick::GameTick;

pub struct GameLog {
    pub schedule_game: ScheduleGame,
    pub ticks: Vec<GameTick>,
}