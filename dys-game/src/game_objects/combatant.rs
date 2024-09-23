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
    pub team: TeamAlignment,
    rigid_body_handle: RigidBodyHandle,
    collider_handle: ColliderHandle,
}

#[derive(Clone)]
pub struct CombatantState {
    current_action: Option<Action>,
    plan: Vec<Action>,
    beliefs: Vec<Belief>,

    on_plate: Option<PlateId>,
    holding_ball: Option<BallId>,
    stunned_by_explosion: bool,
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
            combatant_state: CombatantState { 
                on_plate: None,
                holding_ball: None,
                current_action: None,
                plan: vec![],
                beliefs: vec![],
                stunned_by_explosion: false,
            },
            team,
            rigid_body_handle,
            collider_handle,
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
        
        self.combatant_state.stunned_by_explosion = true;
    }

    pub fn set_on_plate(&mut self, plate_id: PlateId) {
        self.combatant_state.on_plate = Some(plate_id);
    }

    pub fn set_off_plate(&mut self) {
        self.combatant_state.on_plate = None;
    }

    pub fn plate(&self) -> Option<PlateId> {
        self.combatant_state.on_plate
    }

    pub fn pickup_ball(&mut self, ball_id: BallId) {
        self.combatant_state.holding_ball = Some(ball_id);
    }

    pub fn drop_ball(&mut self) {
        self.combatant_state.holding_ball = None;
    }

    pub fn ball(&self) -> Option<BallId> {
        self.combatant_state.holding_ball
    }
}

impl GameObject for CombatantObject {
    // ZJ-TODO: remove
    type GameStateT = ();

    fn rigid_body_handle(&self) -> Option<RigidBodyHandle> {
        Some(self.rigid_body_handle)
    }

    fn collider_handle(&self) -> Option<ColliderHandle> {
        Some(self.collider_handle)
    }
    
    fn change_state(&mut self, current_tick: GameTickNumber, new_state: Self::GameStateT) -> (Self::GameStateT, GameTickNumber) {
        // ZJ-TODO: remove
        ((), 0)
    }

    fn is_dirty(&self) -> bool {
        // ZJ-TODO: remove
        false
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
        &self.combatant_state.beliefs
    }

    fn tick(&mut self, game_state: &mut GameState) -> Vec<SimulationEvent> {
        let mut events = vec![];

        if self.combatant_state.current_action.is_none() {
            if self.combatant_state.plan.is_empty() {
                self.combatant_state.plan = Planner::plan(self, game_state);
            }

            let Some(next_action) = self.combatant_state.plan.pop() else {
                return events;
            };
            
            self.combatant_state.current_action = Some(next_action);
        }

        let mut action = self.combatant_state.current_action.take().expect("failed to get current action");

        tracing::debug!("Executing action {}", action.name());
        events.append(&mut action.tick(self, game_state));

        if !action.is_complete() {
            self.combatant_state.current_action = Some(action);
        } else {
            tracing::debug!("Action {} complete - rewarding completion beliefs", action.name());
            self.combatant_state.beliefs.append(&mut action.completion_beliefs());
        }

        events
    }
}