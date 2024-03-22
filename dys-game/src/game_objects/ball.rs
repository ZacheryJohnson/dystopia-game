use dys_world::combatant::combatant::CombatantId;

use crate::{game_state::Vector3, game_tick::GameTickNumber};

pub type BallId = u16;

pub enum BallState {
    Idle,
    Held { 
        holder_id: CombatantId
    },
    RollingInDirection {
        direction: Vector3,
        velocity: f32
    },
    ThrownAtTarget {
        direction: Vector3,
        velocity: f32,
        thrower_id: CombatantId,
        target_id: CombatantId,
    },
    Explode,
}

pub struct BallObject {
    pub id: BallId,
    pub state: BallState,
    pub state_tick_stamp: GameTickNumber,
    pub world_position: Vector3,
    pub charge: f32,
}

impl BallObject {
    pub fn new(id: BallId, creation_tick: GameTickNumber, position: Vector3) -> BallObject {
        BallObject {
            id,
            state: BallState::Idle,
            state_tick_stamp: creation_tick,
            world_position: position,
            charge: 0.0,
        }
    }
}