use std::sync::{Arc, Mutex};

use dys_world::combatant::combatant::Combatant;
use rapier3d::{dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, ColliderHandle, ColliderSet}, na::Vector3, pipeline::ActiveEvents};

use crate::game_tick::GameTickNumber;

use super::{ball::BallId, game_object::GameObject};

pub type CombatantId = u64;

const COMBATANT_HALF_HEIGHT: f32 = 2.0; // ZJ-TODO: this should be derived from the character's limbs
const COMBATANT_RADIUS: f32 = 0.5; // ZJ-TODO: this should be derived from the character's limbs

#[derive(PartialEq, Eq, Debug)]
pub enum TeamAlignment {
    Home,
    Away,
}

pub struct CombatantObject {
    pub id: CombatantId,
    pub combatant: Arc<Mutex<Combatant>>,
    pub combatant_state: CombatantState,
    pub state_tick_stamp: GameTickNumber,
    pub team: TeamAlignment,
    rigid_body_handle: RigidBodyHandle,
    collider_handle: ColliderHandle,
    pub is_dirty: bool
}

pub enum CombatantState {
    Idle,
    MovingToBall { ball_id: BallId },
    MovingToPosition { position: Vector3<f32> },
    RecoilingFromExplosion {  },
}

impl CombatantObject {
    pub fn new(id: CombatantId, combatant: Arc<Mutex<Combatant>>, position: Vector3<f32>, team: TeamAlignment, rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet) -> CombatantObject {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(position)
            .build();
        
        let collider = ColliderBuilder::capsule_y(COMBATANT_HALF_HEIGHT, COMBATANT_RADIUS)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();

        let rigid_body_handle = rigid_body_set.insert(rigid_body);
        let collider_handle = collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);        
        
        CombatantObject {
            id,
            combatant,
            combatant_state: CombatantState::Idle,
            state_tick_stamp: 0,
            team,
            rigid_body_handle,
            collider_handle,
            is_dirty: false,
        }
    }

    pub fn apply_explosion_force(
        &mut self,
        current_tick: GameTickNumber,
        force_magnitude: f32,
        force_direction: Vector3<f32>,
        rigid_body_set: &mut RigidBodySet)
    {
        let self_rb = rigid_body_set.get_mut(self.rigid_body_handle).expect("failed to get own rigidbody");
        let impulse = force_direction.normalize() * force_magnitude;
        println!("{impulse:?}");
        self_rb.apply_impulse(impulse, true);

        // ZJ-TODO: apply damage to limbs, etc
        
        self.combatant_state = CombatantState::RecoilingFromExplosion {  };
        self.state_tick_stamp = current_tick;
        self.is_dirty = true;
    }
}

impl GameObject for CombatantObject {
    fn rigid_body_handle(&self) -> Option<RigidBodyHandle> {
        Some(self.rigid_body_handle)
    }

    fn collider_handle(&self) -> Option<ColliderHandle> {
        Some(self.collider_handle)
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}