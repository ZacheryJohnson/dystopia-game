pub struct SimulationConfig {
    ticks_per_second: u32,
    seconds_per_half: u32,
    ball_charge_decay_per_tick: f32,
    ball_charge_maximum: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self { 
            ticks_per_second: 10,
            seconds_per_half: 3 * 60,
            ball_charge_decay_per_tick: 2.0,
            ball_charge_maximum: 100.0
        }
    }
}

impl SimulationConfig {
    pub fn ticks_per_second(&self) -> u32 { self.ticks_per_second }
    pub fn seconds_per_half(&self) -> u32 { self.seconds_per_half }
    pub fn ticks_per_half(&self) -> u32 { self.ticks_per_second * self.seconds_per_half }
    pub fn ticks_per_game(&self) -> u32 { self.ticks_per_half() * 2 }
    pub fn ball_charge_decay_per_tick(&self) -> f32 { self.ball_charge_decay_per_tick }
    pub fn ball_charge_maximum(&self) -> f32 { self.ball_charge_maximum }
}