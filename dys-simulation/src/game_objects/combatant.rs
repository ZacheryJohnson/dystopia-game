use std::{fmt::Debug, sync::{Arc, Mutex}};

use dys_world::{arena::plate::PlateId, combatant::instance::CombatantInstance};
use rapier3d::{dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ActiveCollisionTypes, ColliderBuilder, ColliderHandle, ColliderSet}, na::Vector3, pipeline::ActiveEvents};
use rapier3d::na::Isometry3;
use crate::{ai::{action::Action, agent::Agent, belief::Belief, planner}, game_state::GameState, game_tick::GameTickNumber, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::BeliefSet;
use crate::ai::sensor::{FieldOfViewSensor, Sensor};
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
    pub combatant: Arc<Mutex<CombatantInstance>>,
    pub combatant_state: Arc<Mutex<CombatantState>>,
    pub team: TeamAlignment,
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
}

#[derive(Clone, Default, Debug)]
pub struct CombatantState {
    current_action: Option<Action>,
    plan: Vec<Action>,
    pub beliefs: BeliefSet,
    pub field_of_view_sensors: Vec<(u32, FieldOfViewSensor)>,

    pub on_plate: Option<PlateId>,
    pub holding_ball: Option<BallId>,
    pub stunned_by_explosion: bool,
}

impl CombatantObject {
    pub fn new(
        id: CombatantId,
        combatant: Arc<Mutex<CombatantInstance>>,
        position: Vector3<f32>,
        team: TeamAlignment,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet
    ) -> CombatantObject {
        let rigid_body = RigidBodyBuilder::dynamic() // RigidBodyBuilder::kinematic_position_based()
            .translation(position)
            .enabled_rotations(false, true, false)
            .build();
        
        let collider = ColliderBuilder::cuboid(COMBATANT_RADIUS, COMBATANT_HALF_HEIGHT, COMBATANT_RADIUS)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .active_collision_types(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_FIXED | ActiveCollisionTypes::KINEMATIC_KINEMATIC)
            .density(COMBATANT_MASS)
            .position(Isometry3::translation(0.0, COMBATANT_HALF_HEIGHT, 0.0))
            .build();

        let rigid_body_handle = rigid_body_set.insert(rigid_body);
        let collider_handle = collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);

        const SIGHT_DISTANCE: f32 = 50.0;
        let field_of_view_sensor = FieldOfViewSensor::new(
            SIGHT_DISTANCE, collider_handle
        );

        CombatantObject {
            id,
            combatant,
            combatant_state: Arc::new(Mutex::new(CombatantState {
                on_plate: None,
                holding_ball: None,
                current_action: None,
                plan: vec![],
                beliefs: BeliefSet::empty(),
                field_of_view_sensors: vec![(1, field_of_view_sensor)],
                stunned_by_explosion: false,
            })),
            team,
            rigid_body_handle,
            collider_handle,
        }
    }

    pub fn sensors(&self) -> Vec<(u32, impl Sensor)> {
        let combatant_state = self.combatant_state.lock().unwrap();
        combatant_state
            .field_of_view_sensors
            .iter()
            .map(|ref_item| ref_item.to_owned())
            .collect()
    }

    /// Gets the "forward" isometry for the current combatant.
    /// This consists of a translation, which is the origin of the rigid body + the radius of the combatant,
    /// and a rotation, which is the direction the combatant is currently facing.
    /// The rotation will always be strictly around the Y-axis, and the X and Z axes will always be zero.
    pub fn forward_isometry(&self, rigid_body_set: &RigidBodySet) -> Isometry3<f32> {
        let rigid_body = rigid_body_set
            .get(self.rigid_body_handle)
            .unwrap();

        Isometry3::new(
            rigid_body.translation().to_owned(),
            rigid_body.rotation().scaled_axis()
        )
    }

    pub fn set_on_plate(&mut self, plate_id: PlateId) {
        {
            let mut combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.on_plate = Some(plate_id);
            combatant_state.beliefs.add_belief(Belief::OnPlate { combatant_id: self.id, plate_id });
        }
    }

    pub fn set_off_plate(&mut self) {
        {
            let mut combatant_state = self.combatant_state.lock().unwrap();
            let Some(old_plate_id) = combatant_state.on_plate else {
                return;
            };

            combatant_state.on_plate = None;
            combatant_state.beliefs.remove_belief(
                Belief::OnPlate { combatant_id: self.id, plate_id: old_plate_id }
            );
        }
    }

    pub fn plate(&self) -> Option<PlateId> {
        {
            let combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.on_plate
        }
    }

    pub fn pickup_ball(&mut self, ball_id: BallId) {
        {
            let mut combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.holding_ball = Some(ball_id);
        }
    }

    pub fn drop_ball(&mut self) {
        {
            let mut combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.holding_ball = None;
        }
    }

    pub fn ball(&self) -> Option<BallId> {
        {
            let combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.holding_ball
        }
    }
}

impl GameObject for CombatantObject {
    type GameObjectIdT = CombatantId;
    // ZJ-TODO: remove
    type GameStateT = ();

    fn id(&self) -> Self::GameObjectIdT {
        self.id
    }

    fn rigid_body_handle(&self) -> Option<RigidBodyHandle> {
        Some(self.rigid_body_handle)
    }

    fn collider_handle(&self) -> Option<ColliderHandle> {
        Some(self.collider_handle)
    }
    
    fn change_state(&mut self, _: GameTickNumber, _: Self::GameStateT) -> (Self::GameStateT, GameTickNumber) {
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

    fn beliefs(&self) -> BeliefSet {
        let combatant_state = self.combatant_state.lock().unwrap();
        combatant_state.beliefs.to_owned()
    }

    #[tracing::instrument(
        name = "agent::tick",
        fields(
            combatant_id = self.id,
            current_tick = game_state.lock().unwrap().current_tick,
        ),
        skip_all,
        level = "trace")]
    fn tick(
        &mut self,
        game_state: Arc<Mutex<GameState>>,
    ) -> Vec<SimulationEvent> {
        let mut events = vec![];

        let current_plan = {
            let combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.plan.clone()
        };

        let current_beliefs = self.beliefs();
        for action in &current_plan {
            if action.should_interrupt(&current_beliefs) {
                let mut combatant_state = self.combatant_state.lock().unwrap();
                combatant_state.plan.clear();
                combatant_state.current_action = None;
                break;
            }
        }

        let (current_action_is_none, plan_is_empty) = {
            let combatant_state = self.combatant_state.lock().unwrap();
            (combatant_state.current_action.is_none(), combatant_state.plan.is_empty())
        };

        if current_action_is_none {
            if plan_is_empty {
                let new_plan = planner::plan(self, game_state.clone());
                let mut combatant_state = self.combatant_state.lock().unwrap();
                combatant_state.plan = new_plan;
            }

            {
                let mut combatant_state = self.combatant_state.lock().unwrap();
                let Some(next_action) = combatant_state.plan.pop() else {
                    return events;
                };

                combatant_state.current_action = Some(next_action);
            }
        }

        let mut action = {
            let mut combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.current_action.take().expect("failed to get current action")
        };

        if !action.can_perform(&self.beliefs()) {
            tracing::info!("Can no longer perform action; setting to None to replan next tick");
            return vec![];
        }

        tracing::debug!("Executing action {}", action.name());

        let Some(action_result_events) = action.tick(self, game_state) else {
            tracing::debug!("Failed to execute action (the world state may have changed) - setting current action to None to replan next tick");
            return vec![];
        };

        events.extend(action_result_events);

        if !action.is_complete(&self.beliefs()) {
            // The action is not complete - set it to the same action again
            let mut combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.current_action = Some(action);
        } else {
            tracing::debug!("Action {} complete - rewarding completion beliefs", action.name());

            let mut combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.beliefs.add_beliefs(&mut action.completion_beliefs().to_owned());
        }

        events
    }
}

impl Debug for CombatantObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CombatantObject")
            .field("id", &self.id)
            .field("combatant", &self.combatant.lock().unwrap())
            .field("combatant_state", &self.combatant_state)
            .field("team", &self.team)
            .field("rigid_body_handle", &self.rigid_body_handle)
            .field("collider_handle", &self.collider_handle)
            .finish()
    }
}
