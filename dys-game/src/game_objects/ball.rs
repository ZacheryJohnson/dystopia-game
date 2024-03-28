use dys_world::combatant::combatant::CombatantId;
use rapier3d::{dynamics::{CoefficientCombineRule, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, ColliderHandle, ColliderSet}, na::Vector3, pipeline::ActiveEvents};

use crate::game_tick::GameTickNumber;

pub type BallId = u16;

const BALL_RADIUS: f32 = 0.5;
const BALL_RESTITUTION: f32 = 0.2;
const BALL_MASS: f32 = 2.0;

#[derive(Clone)]
pub enum BallState {
    Idle,
    Held { 
        holder_id: CombatantId
    },
    RollingInDirection {
        direction: Vector3<f32>,
        velocity: f32
    },
    ThrownAtTarget {
        direction: Vector3<f32>,
        velocity: f32,
        thrower_id: CombatantId,
        target_id: CombatantId,
    },
    Explode,
}

#[derive(Clone)]
pub struct BallObject {
    pub id: BallId,
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub state: BallState,
    pub state_tick_stamp: GameTickNumber,
    pub charge: f32,
}

impl BallObject {
    pub fn new(id: BallId, creation_tick: GameTickNumber, position: Vector3<f32>, rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet) -> BallObject {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(position)
            .build();
        
        let collider = ColliderBuilder::ball(BALL_RADIUS)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .restitution(BALL_RESTITUTION)
            .restitution_combine_rule(CoefficientCombineRule::Min)
            .density(BALL_MASS)
            .build();

        let rigid_body_handle = rigid_body_set.insert(rigid_body);
        let collider_handle = collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);

        BallObject {
            id,
            rigid_body_handle,
            collider_handle,
            state: BallState::Idle,
            state_tick_stamp: creation_tick,
            charge: 0.0,
        }
    }

    pub fn change_state(&mut self, current_tick: GameTickNumber, new_state: BallState) -> (BallState, GameTickNumber) {
        let old_state = self.state.clone();
        let old_tick_timestamp = self.state_tick_stamp;
        
        self.state = new_state;
        self.state_tick_stamp = current_tick;

        (old_state, old_tick_timestamp)
    }
}