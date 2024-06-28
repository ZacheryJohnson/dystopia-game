use std::{ops::Add, time::Duration};

use crate::simulation::simulation_event::SimulationEvent;

pub type GameTickNumber = u32;

pub struct GameTick {
    pub tick_number: u32,
    pub tick_performance: TickPerformance,
    pub simulation_events: Vec<SimulationEvent>,
    pub(crate) is_halftime: bool,
    pub(crate) is_end_of_game: bool,
    pub(crate) is_scoring_tick: bool,
}

impl GameTick {
    pub fn is_halftime(&self) -> bool { self.is_halftime }
    pub fn is_end_of_game(&self) -> bool { self.is_end_of_game }
    pub fn tick_performance(&self) -> &TickPerformance { &self.tick_performance }

    pub fn perf_string(&self) -> String {
        self.tick_performance.perf_string()
    }
}

#[derive(Clone)]
pub struct TickPerformance {
    pub physics_duration: Duration,
    pub balls_duration: Duration,
    pub combatant_duration: Duration,
    pub scoring_duration: Duration,
    pub tick_duration: Duration,
}

impl TickPerformance {
    pub fn new(
        physics_duration: Duration,
        balls_duration: Duration,
        combatant_duration: Duration,
        scoring_duration: Duration,
        tick_duration: Duration,
    ) -> TickPerformance {
        TickPerformance {
            physics_duration,
            balls_duration,
            combatant_duration,
            scoring_duration,
            tick_duration,
        }
    }

    pub fn perf_string(&self) -> String {
        format!("{} total μs ({}μs phys, {}μs balls, {}μs combatants, {}μs scoring)", 
            self.tick_duration.as_micros(),
            self.physics_duration.as_micros(),
            self.balls_duration.as_micros(),
            self.combatant_duration.as_micros(),
            self.scoring_duration.as_micros())
    }
}

impl Default for TickPerformance {
    fn default() -> Self {
        Self { 
            physics_duration: Default::default(),
            balls_duration: Default::default(),
            combatant_duration: Default::default(),
            scoring_duration: Default::default(),
            tick_duration: Default::default()
        }
    }
}

impl Add for TickPerformance {
    type Output = TickPerformance;

    fn add(self, rhs: Self) -> Self::Output {
        TickPerformance::new(
            self.physics_duration + rhs.physics_duration,
            self.balls_duration + rhs.balls_duration,
            self.combatant_duration + rhs.combatant_duration,
            self.scoring_duration + rhs.scoring_duration,
            self.tick_duration + rhs.tick_duration,
        )
    }
}