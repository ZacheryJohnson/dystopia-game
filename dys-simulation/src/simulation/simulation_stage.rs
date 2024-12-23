use std::time::Duration;
use crate::simulation::simulation_event::SimulationEvent;

pub struct SimulationStage {
    pub execution_duration: Duration,
    pub pending_events: Vec<SimulationEvent>,
}