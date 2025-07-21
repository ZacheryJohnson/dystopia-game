use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Number of ticks per one second of simulation time.
    /// For example, 10 ticks of second means each simulation tick would cover 100 milliseconds
    /// of "real" time.
    /// A higher value will result in a more "accurate" physics sim but results in longer
    /// simulation durations.
    /// 10 ticks per second has been a reasonable default during dev, as it leads to believable
    /// simulations that don't take forever to run, and have the nice side effect of each tick
    /// simulating a clean number of 100 milliseconds.
    /// (vs ticks_per_second=13 being 76.923 milliseconds per tick)
    ticks_per_second: u32,

    /// Number of periods in a game.
    /// A period is an arbitrary-length section of a game (such as halves in soccer, quarters in basketball, etc).
    /// All periods are assumed to be the same length (eg no 2 minute period, then 3 minute period).
    /// If set to zero, a single period will be assumed. (meaning 0 is equivalent to 1)
    /// Cannot be set to zero if game_conclusion_score is zero.
    periods_per_game: u32,

    /// Number of seconds per period.
    /// If periods_per_game is non-zero, this must be non-zero.
    seconds_per_period: u32,

    /// The amount of ball charge increase per tick.
    /// This is linear.
    /// Can be negative, which would represent a ball losing charge while flying.
    ball_charge_increase_per_tick: f32,

    /// The maximum charge a ball can reach.
    /// Must be a non-negative number.
    ball_charge_maximum: f32,

    /// Upon reaching or exceeding this score, the game will end.
    /// This is a static value, and not a mercy rule difference.
    /// If set to zero, the game will only end after time expires.
    /// Cannot be set to zero if periods_per_game is zero.
    game_conclusion_score: u16,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self { 
            ticks_per_second: 10,
            seconds_per_period: 60,
            periods_per_game: 2,
            ball_charge_increase_per_tick: 5.0,
            ball_charge_maximum: 100.0,
            game_conclusion_score: 150,
        }
    }
}

impl SimulationConfig {
    /// Checks that the config is valid.
    pub fn is_valid(&self) -> bool {
        if self.ticks_per_second == 0 {
            tracing::error!("Failed to validate config - ticks per second must be a positive number");
            return false;
        }

        if self.game_conclusion_score == 0 && self.periods_per_game == 0 {
            tracing::error!("Failed to validate config - no maximum score or periods configured");
            return false;
        }

        if self.periods_per_game > 1 && self.seconds_per_period == 0 {
            tracing::error!("Failed to validate config - seconds per period cannot be 0 if periods per game is set");
            return false;
        }

        if self.ball_charge_maximum < 0.0 {
            tracing::error!("Failed to validate config - ball charge maximum cannot be negative");
        }

        true
    }

    pub fn ticks_per_second(&self) -> u32 { self.ticks_per_second }
    pub fn seconds_per_period(&self) -> u32 { self.seconds_per_period }
    pub fn ticks_per_period(&self) -> u32 { self.ticks_per_second * self.seconds_per_period }
    pub fn ticks_per_game(&self) -> u32 { self.ticks_per_period() * self.periods_per_game }
    pub fn ball_charge_increase_per_tick(&self) -> f32 { self.ball_charge_increase_per_tick }
    pub fn ball_charge_maximum(&self) -> f32 { self.ball_charge_maximum }
    pub fn game_conclusion_score(&self) -> u16 { self.game_conclusion_score }
}

#[cfg(test)]
mod tests {
    use crate::simulation::config::SimulationConfig;

    #[test]
    fn test_default_config_is_valid() {
        assert!(SimulationConfig::default().is_valid());
    }
}