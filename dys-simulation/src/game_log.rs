use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use dys_world::matches::instance::MatchInstanceId;
use crate::combatant_statline::CombatantStatline;
use crate::game_state::{GameState, SeedT};
use crate::game_tick::{GameTick, TickPerformance};
use crate::simulation::simulation_event::SimulationEvent;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GameLog {
    match_id: MatchInstanceId,
    seed: SeedT,
    home_score: u16,
    away_score: u16,
    ticks: Vec<GameTick>,
    combatant_statlines: Vec<CombatantStatline>,
    performance: TickPerformance,
}

impl GameLog {
    pub fn from_ticks(ticks: Vec<GameTick>, game_state: Arc<Mutex<GameState>>) -> GameLog {
        let perf = ticks
            .iter()
            .map(|game_tick| game_tick.tick_performance())
            .fold(TickPerformance::default(), |acc_perf, next_perf| acc_perf + next_perf.to_owned());

        let mut combatant_statlines = Vec::new();
        let events = ticks
            .iter()
            .map(|tick| tick.simulation_events.to_owned())
            .flatten()
            .collect::<Vec<_>>();

        let game_state = game_state.lock().unwrap();
        for (combatant_id, _) in &game_state.combatants {
            let points_scored = events
                .iter()
                .filter(|evt| matches!(evt, SimulationEvent::PointsScoredByCombatant { combatant_id: cid, .. } if cid == combatant_id ))
                .map(|evt| {
                    let SimulationEvent::PointsScoredByCombatant { points, ..} = evt else {
                        return 0;
                    };

                    points.to_owned()
                })
                .sum();

            let balls_thrown = events
                .iter()
                .filter(|evt| matches!(evt, SimulationEvent::BallThrownAtEnemy { thrower_id: cid, .. } if cid == combatant_id ))
                .count() as u16;

            let combatants_shoved = events
                .iter()
                .filter(|evt| matches!(evt, SimulationEvent::CombatantShoveForceApplied { shover_combatant_id: cid, .. } if cid == combatant_id ))
                .count() as u16;

            combatant_statlines.push(CombatantStatline {
                combatant_id: *combatant_id,
                points_scored,
                balls_thrown,
                throws_hit: 0, // ZJ-TODO
                combatants_shoved,
            });
        }

        GameLog {
            match_id: game_state.game.match_instance.match_id,
            seed: game_state.seed,
            home_score: game_state.home_points,
            away_score: game_state.away_points,
            ticks,
            combatant_statlines,
            performance: perf,
        }
    }

    pub fn home_score(&self) -> u16 { self.home_score }

    pub fn away_score(&self) -> u16 { self.away_score }

    pub fn ticks(&self) -> &Vec<GameTick> {
        &self.ticks
    }

    pub fn combatant_statlines(&self) -> &Vec<CombatantStatline> {
        &self.combatant_statlines
    }

    pub fn perf_string(&self) -> String {
        self.performance.perf_string()
    }

    pub fn seed(&self) -> SeedT {
        self.seed
    }
}