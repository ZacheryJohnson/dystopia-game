use std::sync::{Arc, Mutex};
use dys_world::arena::navmesh::{ArenaNavmeshNode, ArenaNavmeshPath};
use rapier3d::na::Point3;
use rapier3d::prelude::*;
use crate::{ai::{agent::Agent, strategy::Strategy}, game_state::GameState, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::BeliefSet;
use crate::game_objects::combatant::CombatantId;
use crate::game_objects::game_object::GameObject;
use crate::game_objects::game_object_type::GameObjectType;

pub struct MoveToLocationStrategy {
    is_complete: bool,
    path: ArenaNavmeshPath,
    next_node: Option<ArenaNavmeshNode>,
    self_combatant_id: CombatantId,
    start_location: Option<Point3<f32>>,
    target_game_object: Option<GameObjectType>,
    target_location: Point3<f32>,
    max_ticks: u16,
    dynamic_pathing: bool,
}

impl MoveToLocationStrategy {
    pub fn new(
        self_combatant_id: CombatantId,
        target_location: Point3<f32>,
        max_ticks: u16,
    ) -> MoveToLocationStrategy {
        // Rather than immediately construct a path, we'll wait until actually executing the strategy
        // Otherwise, we'd be making many paths, the majority of which we'd never use

        MoveToLocationStrategy {
            is_complete: false,
            path: ArenaNavmeshPath::empty(),
            next_node: None,
            self_combatant_id,
            start_location: None,
            target_game_object: None,
            target_location,
            max_ticks,
            dynamic_pathing: false,
        }
    }

    pub fn new_with_target_object(
        self_combatant_id: CombatantId,
        target_object: GameObjectType,
        max_ticks: u16,
    ) -> MoveToLocationStrategy {
        MoveToLocationStrategy {
            is_complete: false,
            path: ArenaNavmeshPath::empty(),
            next_node: None,
            self_combatant_id,
            start_location: None,
            target_game_object: Some(target_object),
            target_location: point![0.0, 0.0, 0.0],
            max_ticks,
            dynamic_pathing: false,
        }
    }

    pub fn new_with_target_tracking(
        self_combatant_id: CombatantId,
        target_object: GameObjectType,
    ) -> MoveToLocationStrategy {
        MoveToLocationStrategy {
            is_complete: false,
            path: ArenaNavmeshPath::empty(),
            next_node: None,
            self_combatant_id,
            start_location: None,
            target_game_object: Some(target_object),
            target_location: point![0.0, 0.0, 0.0],
            max_ticks: u16::MAX,
            dynamic_pathing: true,
        }
    }
}

impl MoveToLocationStrategy {
    fn compute_path(
        &mut self,
        game_state: Arc<Mutex<GameState>>,
    ) -> ArenaNavmeshPath {
        let game_state = game_state.lock().unwrap();

        if self.start_location.is_none() {
            let start_location = {
                let (rigid_body_set, _, _) = game_state.physics_sim.sets();
                let combatant_object = game_state
                    .combatants
                    .get(&self.self_combatant_id)
                    .unwrap();
                rigid_body_set.get(combatant_object.rigid_body_handle).unwrap().translation()
            };

            self.start_location = Some((*start_location).into());
        }

        if let Some(target_game_object) = &self.target_game_object {
            self.target_location = match target_game_object {
                GameObjectType::Ball(ball_id) => {
                    let ball_object = game_state.balls.get(ball_id).unwrap();
                    let (rigid_body_set, _, _) = game_state.physics_sim.sets();
                    Point3::from(rigid_body_set
                        .get(ball_object.rigid_body_handle().unwrap())
                        .unwrap()
                        .translation()
                        .to_owned())
                },
                GameObjectType::Combatant(combatant_id) => {
                    let combatant_object = game_state.combatants.get(combatant_id).unwrap();
                    let (rigid_body_set, _, _) = game_state.physics_sim.sets();
                    Point3::from(rigid_body_set
                        .get(combatant_object.rigid_body_handle().unwrap())
                        .unwrap()
                        .translation()
                        .to_owned())
                },
                _ => panic!("unsupported game object type for MoveToLocation strategy"),
            };
        }

        game_state
            .arena_navmesh
            .create_path(self.start_location.unwrap(), self.target_location)
            .unwrap_or(ArenaNavmeshPath::empty())
    }
}

impl Strategy for MoveToLocationStrategy {
    fn name(&self) -> String {
        String::from("Move to Location")
    }

    fn can_perform(&self, _: &BeliefSet) -> bool {
        // If we have no path, we are either uninitialized, or have nowhere to go
        // In either case, we can perform this action, in which this is either a no-op
        // or will initialize the state we need.
        //
        // If we *do* have a path, we can only perform this strategy if we have a node to path to.
        self.path.is_empty() || self.next_node.is_some()
    }

    fn should_interrupt(&self, _: &BeliefSet) -> bool {
        self.max_ticks == 0
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    #[tracing::instrument(
        name = "move_to_location::tick",
        fields(combatant_id = agent.combatant().id),
        skip_all,
        level = "trace"
    )]
    fn tick(
        &mut self,
        agent: &dyn Agent,
        game_state: Arc<Mutex<GameState>>,
    ) -> Option<Vec<SimulationEvent>> {
        let mut events = vec![];
        self.max_ticks = self.max_ticks.checked_sub(1).unwrap_or(0);

        if self.dynamic_pathing {
            // 1. Always finish path to next node if exists
            if self.next_node.is_none() {
                self.path = self.compute_path(game_state.clone());
                self.next_node = self.path.next_node();
            }
        }
        else {
            if self.path.is_empty() && self.next_node.is_none() {
                self.path = self.compute_path(game_state.clone());
                self.next_node = self.path.next_node();
            }
        }

        let (combatant_isometry, unit_resolution) = {
            let game_state = game_state.lock().unwrap();

            let (rigid_body_set, _, _) = game_state.physics_sim.sets();
            let combatant_pos = rigid_body_set
                .get(agent.combatant().rigid_body_handle)
                .unwrap()
                .position()
                .to_owned();

            let unit_resolution = game_state.arena_navmesh.config().unit_resolution;

            (combatant_pos, unit_resolution)
        };

        let mut combatant_position = combatant_isometry.translation.vector;

        let mut total_distance_can_travel_this_tick = agent.combatant().combatant.lock().unwrap().move_speed();

        while total_distance_can_travel_this_tick > 0.0 {
            let Some(next_node) = self.next_node else {
                break;
            };

            let difference_vector = next_node.as_vector() - combatant_position;
            let (updated_position, distance_traveled) = {
                if total_distance_can_travel_this_tick >= difference_vector.magnitude() {
                    (next_node.as_vector(), difference_vector.magnitude())
                } else {
                    let partial_vector = difference_vector.normalize() * total_distance_can_travel_this_tick;

                    (combatant_position + partial_vector, partial_vector.magnitude())
                }
            };

            if distance_traveled == 0.0 {
                total_distance_can_travel_this_tick = 0.0;
            } else {
                combatant_position = updated_position;
                total_distance_can_travel_this_tick = (total_distance_can_travel_this_tick - distance_traveled).max(0.0);
            }

            let distance_from_node = (next_node.as_vector() - combatant_position).magnitude();
            if distance_from_node == 0.0 {
                if self.dynamic_pathing {
                    self.path = self.compute_path(game_state.clone());
                    // pop the first node - it's where we're already standing
                    // ZJ-TODO: fix this
                    let _ = self.path.next_node();
                    self.start_location = Some(combatant_position.into());
                }

                if let Some(next_node) = self.path.next_node() {
                    self.next_node = Some(next_node);
                } else {
                    break;
                }
            }
        }

        let is_at_target = (self.target_location - combatant_position).coords.magnitude() <= unit_resolution;
        if is_at_target || self.next_node.is_none() {
            self.is_complete = true;
        }

        events.push(SimulationEvent::CombatantPositionUpdate { 
            combatant_id: agent.combatant().id,
            position: combatant_position,
        });

        Some(events)
    }
}