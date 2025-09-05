use serde::{Deserialize, Serialize};
use dys_simulation::game_log::GameLog;
use dys_simulation::game_tick::GameTickNumber;
use dys_simulation::simulation::simulation_event::SimulationEvent;
use dys_world::combatant::instance::CombatantInstanceId;
use crate::game_stat::{GameStat, GameStatPointsScored};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CombatantStatline {
    pub combatant_id: CombatantInstanceId,
    pub points_scored: u8,
    pub balls_thrown: u16,
    pub throws_hit: u16,
    pub combatants_shoved: u16,
}

impl CombatantStatline {
    fn combatant_statline_from_game_log(
        game_log: &GameLog,
        combatant_id: CombatantInstanceId,
        through_tick: Option<GameTickNumber>,
    ) -> CombatantStatline {
        let events = game_log
            .ticks()
            .iter()
            .filter(|tick| tick.tick_number <= through_tick.unwrap_or(GameTickNumber::MAX))
            .flat_map(|tick| tick.simulation_events.clone())
            .collect::<Vec<_>>();

        let points_scored = GameStatPointsScored
            .calculate(combatant_id, &events)
            .unwrap_or_default() as u8;

        // ZJ-TODO: balls thrown at teammates may hit enemies and explode
        //          this would be a "hit", but would not count as a "throw"
        let balls_thrown = events
            .iter()
            .filter(|evt| matches!(evt, SimulationEvent::BallThrownAtEnemy { thrower_id: cid, .. } if *cid == combatant_id ))
            .count() as u16;

        let throws_hit = events
            .iter()
            .filter(|evt| matches!(evt, SimulationEvent::BallCollisionEnemy { thrower_id: cid, .. } if *cid == combatant_id ))
            .count() as u16;

        let combatants_shoved = events
            .iter()
            .filter(|evt| matches!(evt, SimulationEvent::CombatantShoveForceApplied { shover_combatant_id: cid, .. } if *cid == combatant_id ))
            .count() as u16;

        CombatantStatline {
            combatant_id,
            points_scored,
            balls_thrown,
            throws_hit,
            combatants_shoved,
        }
    }

    /// Parses statlines for all combatants in a game log.
    pub fn from_game_log(game_log: &GameLog) -> Vec<CombatantStatline> {
        game_log
            .combatants()
            .iter()
            .map(|combatant_id| {
                CombatantStatline::combatant_statline_from_game_log(
                    game_log,
                    *combatant_id,
                    None
                )
            })
            .collect::<Vec<_>>()
    }

    /// Parses statlines for all combatants in a game log, stopping after the provided tick.
    /// For example, if a combatant had 40 points as of tick 100, but 42 points as of tick 101,
    /// `from_game_log_through_tick(&game_log, 100)` would return a statline of 40 points.
    pub fn from_game_log_through_tick(
        game_log: &GameLog,
        through_tick: GameTickNumber,
    ) -> Vec<CombatantStatline> {
        game_log
            .combatants()
            .iter()
            .map(|combatant_id| {
                CombatantStatline::combatant_statline_from_game_log(
                    game_log,
                    *combatant_id,
                    Some(through_tick),
                )
            })
            .collect::<Vec<_>>()
    }
}