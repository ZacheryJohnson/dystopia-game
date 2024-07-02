use std::sync::{Arc, Mutex};

use dys_world::{arena::plate::PlateId, combatant::combatant::Combatant};
use rapier3d::{dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, ColliderHandle, ColliderSet}, na::Vector3, pipeline::ActiveEvents};

use crate::game_tick::GameTickNumber;

use super::{ball::BallId, game_object::GameObject};

pub type CombatantId = u64;

const COMBATANT_HALF_HEIGHT: f32 = 2.0; // ZJ-TODO: this should be derived from the character's limbs
const COMBATANT_RADIUS: f32 = 0.5; // ZJ-TODO: this should be derived from the character's limbs

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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
    is_dirty: bool,
    on_plate: Option<PlateId>,
    holding_ball: Option<BallId>,
}

#[derive(Clone)]
pub enum CombatantState {
    Idle,
    MovingToBall { ball_id: BallId },
    MovingToPlate { plate_id: PlateId },
    RecoilingFromExplosion {},
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
            on_plate: None,
            holding_ball: None
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
        self_rb.apply_impulse(impulse, true);

        // ZJ-TODO: apply damage to limbs, etc
        
        self.change_state(current_tick, CombatantState::RecoilingFromExplosion {});
    }

    pub fn set_on_plate(&mut self, plate_id: PlateId) {
        self.on_plate = Some(plate_id);
    }

    pub fn set_off_plate(&mut self) {
        self.on_plate = None;
    }

    pub fn plate(&self) -> Option<PlateId> {
        self.on_plate
    }

    pub fn pickup_ball(&mut self, ball_id: BallId) {
        self.holding_ball = Some(ball_id);
    }

    pub fn drop_ball(&mut self) {
        self.holding_ball = None;
    }

    pub fn ball(&self) -> Option<BallId> {
        self.holding_ball
    }
}

impl GameObject for CombatantObject {
    type GameStateT = CombatantState;

    fn rigid_body_handle(&self) -> Option<RigidBodyHandle> {
        Some(self.rigid_body_handle)
    }

    fn collider_handle(&self) -> Option<ColliderHandle> {
        Some(self.collider_handle)
    }
    
    fn change_state(&mut self, current_tick: GameTickNumber, new_state: Self::GameStateT) -> (Self::GameStateT, GameTickNumber) {
        let old_state = self.combatant_state.clone();
        let old_tick_timestamp = self.state_tick_stamp;
        
        self.combatant_state = new_state;
        self.state_tick_stamp = current_tick;
        self.is_dirty = true;

        (old_state, old_tick_timestamp)
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}