use std::sync::{Arc, Mutex};

use dys_world::{arena::plate::PlateId, combatant::combatant::Combatant};
use rapier3d::{dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ActiveCollisionTypes, ColliderBuilder, ColliderHandle, ColliderSet}, na::Vector3, pipeline::ActiveEvents};

use crate::{ai::{action::Action, agent::Agent, belief::Belief, planner::Planner}, game_state::GameState, game_tick::GameTickNumber, simulation::simulation_event::SimulationEvent};

use super::{ball::BallId, game_object::GameObject};

pub type CombatantId = u64;

const COMBATANT_HALF_HEIGHT: f32 = 2.0; // ZJ-TODO: this should be derived from the character's limbs
const COMBATANT_RADIUS: f32 = 0.5; // ZJ-TODO: this should be derived from the character's limbs
const COMBATANT_MASS: f32 = 150.0;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum TeamAlignment {
    Home,
    Away,
}

#[derive(Clone)]
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
    current_action: Option<Action>,
    plan: Vec<Action>,
    beliefs: Vec<Belief>,
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
        let rigid_body = RigidBodyBuilder::dynamic() // RigidBodyBuilder::kinematic_position_based()
            .translation(position)
            .build();
        
        let collider = ColliderBuilder::capsule_y(COMBATANT_HALF_HEIGHT, COMBATANT_RADIUS)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .active_collision_types(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_FIXED | ActiveCollisionTypes::KINEMATIC_KINEMATIC)
            .density(COMBATANT_MASS)
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
            holding_ball: None,
            current_action: None,
            plan: vec![],
            beliefs: vec![]
        }
    }

    pub fn apply_explosion_force(
        &mut self,
        current_tick: GameTickNumber,
        force_magnitude: f32,
        force_direction: Vector3<f32>,
        rigid_body_set: &mut RigidBodySet)
    {
        // ZJ-TODO: rework; when we moved to kinematic rigid bodies, we can't apply impulses any more

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

impl Agent for CombatantObject {
    fn combatant(&self) -> &CombatantObject {
        self
    }

    fn combatant_mut(&mut self) -> &mut CombatantObject {
        self
    }

    fn beliefs(&self) -> &Vec<Belief> {
        &self.beliefs
    }

    fn tick(&mut self, game_state: &mut GameState) -> Vec<SimulationEvent> {
        let mut events = vec![];

        if self.current_action.is_none() {
            if self.plan.is_empty() {
                self.plan = Planner::plan(self, game_state);
            }

            let Some(next_action) = self.plan.pop() else {
                return events;
            };
            
            self.current_action = Some(next_action);
        }

        let mut action = self.current_action.take().expect("failed to get current action");

        tracing::debug!("Executing action {}", action.name());
        events.append(&mut action.tick(self, game_state));

        if !action.is_complete() {
            self.current_action = Some(action);
        } else {
            tracing::debug!("Action {} complete - rewarding completion beliefs", action.name());
            self.beliefs.append(&mut action.completion_beliefs());
        }

        events
    }
}