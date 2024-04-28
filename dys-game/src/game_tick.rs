use std::time::Duration;

use crate::simulation::simulation_event::SimulationEvent;

pub type GameTickNumber = u32;

pub struct GameTick {
    pub tick_number: u32,
    pub physics_duration: Duration,
    pub balls_duration: Duration,
    pub combatant_duration: Duration,
    pub tick_duration: Duration,
    pub simulation_events: Vec<SimulationEvent>,
    pub(crate) is_halftime: bool,
    pub(crate) is_end_of_game: bool,
}

impl GameTick {
    pub fn is_halftime(&self) -> bool { self.is_halftime }
    pub fn is_end_of_game(&self) -> bool { self.is_end_of_game }

    pub fn perf_string(&self) -> String {
        let total_duration_micros: u128 = (self.physics_duration + self.balls_duration + self.combatant_duration).as_micros();
        
        format!("{} total μs ({}μs phys, {}μs balls, {}μs combatants)", 
            total_duration_micros,
            self.physics_duration.as_micros(),
            self.balls_duration.as_micros(),
            self.combatant_duration.as_micros())
    }
}