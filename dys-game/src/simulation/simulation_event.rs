use crate::game_objects::{ball::BallId, combatant::CombatantId};

pub enum SimulationEvent {
    BallCollisionEnemy{ thrower_id: CombatantId, target_id: CombatantId, ball_id: BallId },
}