use crate::game_objects::{ball::BallId, combatant::CombatantId};

#[derive(Debug)]
pub enum SimulationEvent {
    /// A ball has been thrown targeting an enemy
    BallThrownAtEnemy { thrower_id: CombatantId, enemy_id: CombatantId, ball_id: BallId },

    /// A ball has been thrown targeting a teammate
    BallThrownAtTeammate { thrower_id: CombatantId, teammate_id: CombatantId, ball_id: BallId },

    /// A ball has collided with an enemy 
    BallCollisionEnemy { thrower_id: CombatantId, enemy_id: CombatantId, ball_id: BallId },

    /// A ball has collided with the ground, defusing it
    BallCollisionArena { thrower_id: CombatantId, original_target_id: CombatantId, ball_id: BallId }
}