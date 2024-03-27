use std::time::Duration;

pub type GameTickNumber = u32;

pub struct GameTick {
    pub tick_number: u32,
    pub physics_duration: Duration,
    pub balls_duration: Duration,
    pub tick_duration: Duration,
    pub(crate) is_halftime: bool,
    pub(crate) is_end_of_game: bool,
}

impl GameTick {
    pub fn is_halftime(&self) -> bool { self.is_halftime }
    pub fn is_end_of_game(&self) -> bool { self.is_end_of_game }
}