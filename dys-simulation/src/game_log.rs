use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use dys_world::combatant::instance::CombatantInstanceId;
use dys_world::games::instance::GameInstanceId;
use crate::game_state::{GameState, SeedT};
use crate::game_tick::{GameTick, TickPerformance};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GameLog {
    game_id: GameInstanceId,
    seed: SeedT,
    home_score: u16,
    away_score: u16,
    ticks: Vec<GameTick>,
    performance: TickPerformance,
    combatants: Vec<CombatantInstanceId>,
}

impl GameLog {
    pub fn from_ticks(ticks: Vec<GameTick>, game_state: Arc<Mutex<GameState>>) -> GameLog {
        let perf = ticks
            .iter()
            .map(|game_tick| game_tick.tick_performance())
            .fold(TickPerformance::default(), |acc_perf, next_perf| acc_perf + next_perf.to_owned());

        let game_state = game_state.lock().unwrap();

        let combatants = game_state.combatants.keys().cloned().collect();

        GameLog {
            game_id: game_state.game.game_instance.game_id,
            seed: game_state.seed,
            home_score: game_state.home_points,
            away_score: game_state.away_points,
            ticks,
            performance: perf,
            combatants,
        }
    }

    pub fn home_score(&self) -> u16 { self.home_score }

    pub fn away_score(&self) -> u16 { self.away_score }

    pub fn ticks(&self) -> &Vec<GameTick> {
        &self.ticks
    }

    pub fn perf_string(&self) -> String {
        self.performance.perf_string()
    }

    pub fn seed(&self) -> SeedT {
        self.seed
    }

    pub fn combatants(&self) -> &Vec<CombatantInstanceId> {
        &self.combatants
    }
}