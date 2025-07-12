use std::{fmt::Debug, sync::{Arc, Mutex}};
use rand::RngCore;
use dys_world::{arena::plate::PlateId, combatant::instance::CombatantInstance};
use rapier3d::{dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ActiveCollisionTypes, ColliderBuilder, ColliderHandle, ColliderSet}, na::Vector3, pipeline::ActiveEvents};
use rapier3d::na::Isometry3;
use rapier3d::prelude::*;
use dys_satisfiable::{SatisfiabilityTest, SatisfiableField};
use dys_world::attribute::attribute_type::AttributeType;
use crate::{ai::{action::Action, agent::Agent, belief::Belief, planner}, game_state::GameState, game_tick::GameTickNumber, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::SatisfiableBelief;
use crate::ai::beliefs::belief_set::BeliefSet;
use crate::ai::sensor::Sensor;
use crate::ai::sensors::field_of_view::FieldOfViewSensor;
use crate::ai::sensors::proximity::ProximitySensor;
use crate::simulation::simulation_event::PendingSimulationEvent;
use crate::simulation::simulation_event::SimulationEvent::BroadcastBelief;
use super::{ball::BallId, game_object::GameObject};

pub type CombatantId = u64;

const COMBATANT_HALF_HEIGHT: f32 = 1.0; // ZJ-TODO: this should be derived from the character's limbs
const COMBATANT_RADIUS: f32 = 0.5; // ZJ-TODO: this should be derived from the character's limbs
const COMBATANT_MASS: f32 = 100.0;

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
    pub completed_action: Option<Action>,
    pub current_action: Option<Action>,
    pub plan: Vec<Action>,
    pub beliefs: BeliefSet,
    pub sensors: Vec<(u32, Box<dyn Sensor>)>,

    // ZJ-TODO: this should instead be a set of temporary limb modifiers
    pub damage: f32,
    pub on_plate: Option<PlateId>,
    pub holding_ball: Option<BallId>,
    pub stunned: bool,
}

impl CombatantObject {
    pub fn new(
        id: CombatantId,
        combatant: Arc<Mutex<CombatantInstance>>,
        position: Vector3<f32>,
        rotation: Vector3<f32>,
        team: TeamAlignment,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet
    ) -> CombatantObject {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(position)
            .rotation(rotation)
            .enabled_rotations(false, true, false)
            .ccd_enabled(true) // enable CCD to ensure we don't phase through walls
            .build();
        
        let collider = ColliderBuilder::cuboid(COMBATANT_RADIUS, COMBATANT_HALF_HEIGHT, COMBATANT_RADIUS)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .active_collision_types(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_FIXED | ActiveCollisionTypes::KINEMATIC_KINEMATIC)
            .mass(COMBATANT_MASS)
            .position(Isometry3::translation(0.0, COMBATANT_HALF_HEIGHT, 0.0))
            .build();

        let rigid_body_handle = rigid_body_set.insert(rigid_body);
        let collider_handle = collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);

        const SIGHT_DISTANCE: f32 = 70.0;
        let field_of_view_sensor = FieldOfViewSensor::new(
            SIGHT_DISTANCE, id, collider_handle
        );

        // ZJ-TODO: determine this based on limbs
        let ball_pickup_range_proximity_radius = COMBATANT_RADIUS * 2.0;
        let ball_pickup_range_proximity_sensor = ProximitySensor::new(
            id, COMBATANT_HALF_HEIGHT * 2.0, ball_pickup_range_proximity_radius, collider_handle
        );

        let ball_danger_proximity_range = 2.0_f32;
        let ball_danger_proximity_radius = (COMBATANT_RADIUS * 2.0) + ball_danger_proximity_range;
        let mut ball_danger_proximity_sensor = ProximitySensor::new(
            id, COMBATANT_HALF_HEIGHT * 2.0, ball_danger_proximity_radius, collider_handle
        );

        ball_danger_proximity_sensor.set_yields_beliefs(false);

        CombatantObject {
            id,
            combatant,
            combatant_state: Arc::new(Mutex::new(CombatantState {
                on_plate: None,
                holding_ball: None,
                current_action: None,
                completed_action: None,
                plan: vec![],
                beliefs: BeliefSet::empty(),
                sensors: vec![
                    (1, Box::new(field_of_view_sensor)),
                    (2, Box::new(ball_pickup_range_proximity_sensor)),
                    (3, Box::new(ball_danger_proximity_sensor)),
                ],
                damage: 0.0,
                stunned: false,
            })),
            team,
            rigid_body_handle,
            collider_handle,
        }
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
            rigid_body.translation().to_owned() + vector![0.0, COMBATANT_HALF_HEIGHT, 0.0],
            rigid_body.rotation().scaled_axis()
        )
    }

    pub fn radius(&self) -> f32 {
        // ZJ-TODO: read from combatant object
        COMBATANT_RADIUS
    }

    pub fn weight(&self) -> f32 {
        COMBATANT_MASS
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

    pub fn set_stunned(&mut self, stunned: bool) {
        let mut combatant_state = self.combatant_state.lock().unwrap();
        combatant_state.stunned = stunned;

        // ZJ_TODO: make this a function `invalidate_plan`
        //          we also do this in the action planner
        if stunned {
            combatant_state.plan.clear();
            combatant_state.current_action = None;
        }
    }

    pub fn is_stunned(&self) -> bool {
        self.combatant_state.lock().unwrap().stunned
    }

    pub fn apply_damage(&mut self, damage: f32) {
        let mut combatant_state = self.combatant_state.lock().unwrap();
        combatant_state.damage += damage;
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
        level = "debug")]
    fn tick(
        &mut self,
        game_state: Arc<Mutex<GameState>>,
    ) -> Vec<PendingSimulationEvent> {
        let mut events = vec![];

        let combatant_damage = {
            let combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.damage
        };

        if self.is_stunned() {
            let random_value = game_state.lock().unwrap().rng.next_u32() % (1000 + combatant_damage.floor() as u32);
            let constitution = self
                .combatant
                .lock()
                .unwrap()
                .get_attribute_value(&AttributeType::Constitution)
                .unwrap_or(0.0);

            if constitution >= random_value as f32 {
                let mut game_state = game_state.lock().unwrap();
                let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();
                rigid_body_set.get_mut(self.rigid_body_handle).unwrap().set_linvel(
                    Vector3::zeros(),
                    true
                );

                events.push(PendingSimulationEvent(
                    SimulationEvent::CombatantStunned {
                        combatant_id: self.id,
                        start: false,
                    }
                ));
            }

            return events;
        }

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

        tracing::debug!("executing action {}", action.name());

        let Some(action_result_events) = action.tick(self, game_state.clone()) else {
            tracing::debug!("Failed to execute action {} (the world state may have changed) - setting current action to None to replan next tick", action.name());
            return vec![];
        };

        events.extend(action_result_events);

        if !action.is_complete(&self.beliefs()) {
            // The action is not complete - set it to the same action again
            let mut combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.completed_action = None;
            combatant_state.current_action = Some(action);
        } else {
            tracing::debug!("Action {} complete - rewarding completion beliefs", action.name());

            let mut combatant_state = self.combatant_state.lock().unwrap();
            combatant_state.completed_action = Some(action.to_owned());
            combatant_state.beliefs.add_beliefs(action.completion_beliefs());

            for broadcast_belief in action.broadcast_beliefs() {
                events.push(PendingSimulationEvent(
                    BroadcastBelief {
                        from_combatant_id: self.id,
                        belief: broadcast_belief.to_owned(),
                    }
                ));
            }

            // ZJ-TODO: HACK: yuck
            for belief in action.completion_beliefs() {
                if SatisfiableBelief::BallCaught()
                    .combatant_id(SatisfiableField::Exactly(self.combatant().id))
                    .satisfied_by(*belief) {
                    let Belief::BallCaught {
                        combatant_id, thrower_id, ball_id
                    } = belief else {
                        panic!("how does this happen");
                    };
                    events.push(PendingSimulationEvent(
                        SimulationEvent::ThrownBallCaught {
                            thrower_id: *thrower_id,
                            catcher_id: *combatant_id,
                            ball_id: *ball_id,
                        }
                    ));
                }
            }

            for consumed_belief in action.consumed_beliefs() {
                tracing::debug!("Consuming beliefs satisfying {consumed_belief:?}");
                combatant_state.beliefs.remove_beliefs_by_test(consumed_belief);
            }
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
