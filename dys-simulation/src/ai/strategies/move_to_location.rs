use std::sync::{Arc, Mutex};
use dys_world::arena::navmesh::{ArenaNavmeshNode, ArenaNavmeshPath};
use rapier3d::na::Point3;

use crate::{ai::{agent::Agent, strategy::Strategy}, game_state::GameState, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::BeliefSet;

pub struct MoveToLocationStrategy {
    is_complete: bool,
    path: ArenaNavmeshPath,
    next_node: Option<ArenaNavmeshNode>,
    start_location: Point3<f32>,
    target_location: Point3<f32>,
}

impl MoveToLocationStrategy {
    pub fn new(
        start_location: Point3<f32>,
        target_location: Point3<f32>,
        _: Arc<Mutex<GameState>>,
    ) -> MoveToLocationStrategy {
        // Rather than immediately construct a path, we'll wait until actually executing the strategy
        // Otherwise, we'd be making many paths, the majority of which we'd never use

        MoveToLocationStrategy {
            is_complete: false,
            path: ArenaNavmeshPath::empty(),
            next_node: None,
            start_location,
            target_location,
        }
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
        false
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

        if self.path.is_empty() && self.next_node.is_none() {
            self.path = {
                let game_state = game_state.lock().unwrap();
                game_state
                    .arena_navmesh
                    .create_path(self.start_location, self.target_location)
                    .unwrap_or(ArenaNavmeshPath::empty())
            };

            self.next_node = self.path.next_node();
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