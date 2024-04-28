use rapier3d::na::Vector3;

use crate::game_objects::{ball::BallId, combatant::CombatantId};

#[derive(Debug)]
pub enum SimulationEvent {
    /// A ball has moved through the world
    BallPositionUpdate { ball_id: BallId, position: Vector3<f32> },

    /// A combatant has moved through the world
    CombatantPositionUpdate { combatant_id: CombatantId, position: Vector3<f32> },

    /// A ball has been thrown targeting an enemy
    BallThrownAtEnemy { thrower_id: CombatantId, enemy_id: CombatantId, ball_id: BallId },

    /// A ball has been thrown targeting a teammate
    BallThrownAtTeammate { thrower_id: CombatantId, teammate_id: CombatantId, ball_id: BallId },

    /// A ball has collided with an enemy 
    BallCollisionEnemy { thrower_id: CombatantId, enemy_id: CombatantId, ball_id: BallId },

    /// A ball has collided with the ground, defusing it
    BallCollisionArena { thrower_id: CombatantId, original_target_id: CombatantId, ball_id: BallId },

    /// A ball has exploded
    BallExplosion { ball_id: BallId, charge: f32 },

    /// A ball explosion has applied explosion force to a combatant
    BallExplosionForceApplied { ball_id: BallId, combatant_id: CombatantId, force_magnitude: f32, force_direction: Vector3<f32> }
}