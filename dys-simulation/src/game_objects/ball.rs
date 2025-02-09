use rapier3d::{dynamics::{CoefficientCombineRule, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, ColliderHandle, ColliderSet}, na::Vector3, pipeline::ActiveEvents};
use rapier3d::na::Isometry3;
use crate::game_objects::combatant::CombatantId;
use crate::game_tick::GameTickNumber;

use super::game_object::GameObject;

pub type BallId = u16;

const BALL_RADIUS: f32 = 0.5;
const BALL_RESTITUTION: f32 = 0.2;
const BALL_MASS: f32 = 2.0;

#[derive(Clone, Debug)]
pub enum BallState {
    Idle,
    Held { 
        holder_id: CombatantId
    },
    ThrownAtTarget {
        direction: Vector3<f32>,
        thrower_id: CombatantId,
        target_id: CombatantId,
    },
    Explode,
}

#[derive(Clone)]
pub struct BallObject {
    pub id: BallId,
    rigid_body_handle: RigidBodyHandle,
    collider_handle: ColliderHandle,
    pub state: BallState,
    pub state_tick_stamp: GameTickNumber,
    pub charge: f32,
    pub is_dirty: bool,
    pub held_by: Option<CombatantId>,
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
            is_dirty: false,
            held_by: None,
        }
    }

    pub fn set_held_by(&mut self, combatant_id: Option<CombatantId>, current_tick: GameTickNumber) {
        if !matches!(self.state, BallState::Idle) {
            return;
        }

        self.held_by = combatant_id;

        if let Some(id) = combatant_id {
            self.change_state(current_tick, BallState::Held { holder_id: id });
            self.charge = 30.0;
        } else {
            self.change_state(current_tick, BallState::Idle);
        }
    }
}

impl GameObject for BallObject {
    type GameObjectIdT = BallId;
    type GameStateT = BallState;

    fn id(&self) -> Self::GameObjectIdT {
        self.id
    }

    fn rigid_body_handle(&self) -> Option<RigidBodyHandle> {
        Some(self.rigid_body_handle)
    }

    fn collider_handle(&self) -> Option<ColliderHandle> {
        Some(self.collider_handle)
    }

    fn change_state(&mut self, current_tick: GameTickNumber, new_state: BallState) -> (BallState, GameTickNumber) {
        let old_state = self.state.clone();
        let old_tick_timestamp = self.state_tick_stamp;
        
        self.state = new_state;
        self.state_tick_stamp = current_tick;
        self.is_dirty = true;

        (old_state, old_tick_timestamp)
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}