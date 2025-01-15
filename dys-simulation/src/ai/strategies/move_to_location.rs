use std::sync::{Arc, Mutex};
use dys_world::arena::navmesh::{ArenaNavmeshNode, ArenaNavmeshPath};
use rapier3d::na::Point3;

use crate::{ai::{agent::Agent, strategy::Strategy}, game_state::GameState, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::BeliefSet;
use crate::simulation::simulation_event::PendingSimulationTick;

pub struct MoveToLocationStrategy {
    is_complete: bool,
    path: ArenaNavmeshPath,
    next_node: Option<ArenaNavmeshNode>,
    target_location: Point3<f32>,
}

impl MoveToLocationStrategy {
    pub fn new(
        start_location: Point3<f32>,
        target_location: Point3<f32>,
        game_state: Arc<Mutex<GameState>>,
    ) -> MoveToLocationStrategy {
        let mut path = {
            let game_state = game_state.lock().unwrap();
            game_state
                .arena_navmesh
                .create_path(start_location, target_location)
                .unwrap_or(ArenaNavmeshPath::empty())
        };

        let next_node = path.next_node();

        MoveToLocationStrategy {
            is_complete: false,
            path,
            next_node,
            target_location,
        }
    }
}

impl Strategy for MoveToLocationStrategy {
    fn name(&self) -> String {
        String::from("Move to Location")
    }

    fn can_perform(&self, _: &BeliefSet) -> bool {
        self.next_node.is_some()
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    #[tracing::instrument(name = "move_to_location::tick", fields(combatant_id = agent.combatant().id), skip_all, level = "trace")]
    fn tick(
        &mut self,
        agent: &dyn Agent,
        game_state: Arc<Mutex<GameState>>,
    ) -> Option<Vec<SimulationEvent>> {
        let mut events = vec![];

        let (mut new_combatant_position, unit_resolution) = {
            let game_state = game_state.lock().unwrap();

            let (rigid_body_set, _, _) = game_state.physics_sim.sets();
            let combatant_pos = rigid_body_set
                .get(agent.combatant().rigid_body_handle)
                .unwrap()
                .translation()
                .to_owned();

            let unit_resolution = game_state.arena_navmesh.config().unit_resolution;

            (combatant_pos, unit_resolution)
        };

        // ZJ-TODO: HACK: y coordinate is wonky
        //                ignore whatever we see initially and just maintain the combatant's y-pos
        self.target_location.y = new_combatant_position.y;

        // ZJ-TODO: this is broken somehow - even with different move speeds, combatants are still moving uniform distances
        let mut total_distance_can_travel_this_tick = agent.combatant().combatant.lock().unwrap().move_speed();

        while total_distance_can_travel_this_tick > 0.0 {
            let Some(next_node) = self.next_node else {
                break;
            };
    
            let lerp_distance = total_distance_can_travel_this_tick.clamp(0.0, unit_resolution);
            new_combatant_position = new_combatant_position.lerp(&next_node.as_vector(), lerp_distance);
            total_distance_can_travel_this_tick = (total_distance_can_travel_this_tick - unit_resolution).max(0.0);

            let distance_from_node = (next_node.as_vector() - new_combatant_position).magnitude();
            if distance_from_node == 0.0 {
                if let Some(next_node) = self.path.next_node() {
                    self.next_node = Some(next_node);
                } else {
                    break;
                }
            }
        }

        // ZJ-TODO: HACK: y coordinate is wonky
        //                ignore whatever we see initially and just maintain the combatant's y-pos
        new_combatant_position.y = self.target_location.y;
        let is_at_target = (self.target_location - new_combatant_position).coords.magnitude() <= unit_resolution;
        if is_at_target {
            self.is_complete = true;
        }

        events.push(SimulationEvent::CombatantPositionUpdate { 
            combatant_id: agent.combatant().id,
            position: new_combatant_position
        });

        Some(events)
    }
}