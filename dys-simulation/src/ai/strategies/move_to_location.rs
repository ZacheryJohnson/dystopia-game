use dys_world::arena::navmesh::{ArenaNavmeshNode, ArenaNavmeshPath};
use rapier3d::na::Point3;

use crate::{ai::{agent::Agent, belief::Belief, strategy::Strategy}, game_state::GameState, simulation::simulation_event::SimulationEvent};

pub struct MoveToLocationStrategy {
    is_complete: bool,
    path: ArenaNavmeshPath,
    next_node: Option<ArenaNavmeshNode>,
    target_location: Point3<f32>,
}

impl MoveToLocationStrategy {
    pub fn new(start_location: Point3<f32>, target_location: Point3<f32>, game_state: &GameState) -> MoveToLocationStrategy {
        let mut path = game_state
            .arena_navmesh
            .create_path(start_location, target_location)
            .unwrap_or(ArenaNavmeshPath::new(vec![]));
        let next_node = path.next();

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

    fn can_perform(&self, _: &[Belief]) -> bool {
        self.next_node.is_some()
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    #[tracing::instrument(name = "move_to_location::tick", fields(combatant_id = agent.combatant().id), skip_all, level = "trace")]
    fn tick(
        &mut self,
        agent: &mut dyn Agent,
        game_state: &mut GameState,
    ) -> Option<Vec<SimulationEvent>> {
        let mut events = vec![];

        let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();
        let combatant_rb = rigid_body_set.get_mut(agent.combatant().rigid_body_handle).unwrap();
        let combatant_position = combatant_rb.translation();

        // ZJ-TODO: HACK: y coordinate is wonky
        //                ignore whatever we see initially and just maintain the combatant's y-pos
        self.target_location.y = combatant_position.y;

        let mut total_distance_can_travel_this_tick = agent.combatant().combatant.lock().unwrap().move_speed();

        let mut new_combatant_position = combatant_position.to_owned();

        let unit_resolution = game_state.arena_navmesh.config().unit_resolution;

        while total_distance_can_travel_this_tick >= unit_resolution {
            if self.next_node.is_none() {
                let next_node = self.path.next();
                if next_node.is_none() {
                    break;
                }
                
                self.next_node = next_node;
            }

            let next_node = self.next_node.unwrap();
    
            let lerp_distance = (total_distance_can_travel_this_tick - unit_resolution).clamp(0.0, unit_resolution);
            new_combatant_position = new_combatant_position.lerp(&next_node.as_vector(), lerp_distance);
            total_distance_can_travel_this_tick = (total_distance_can_travel_this_tick - unit_resolution).max(0.0);

            let distance_from_node = (next_node.as_vector() - new_combatant_position).magnitude();
            if distance_from_node == 0.0 {
                self.next_node = None;
            }
        }

        let is_at_target = (self.target_location - new_combatant_position).coords.magnitude() <= unit_resolution;
        if is_at_target {
            self.is_complete = true;
        }
    
        combatant_rb.set_translation(new_combatant_position, true);
        //combatant_rb.set_next_kinematic_translation(new_combatant_position);

        events.push(SimulationEvent::CombatantPositionUpdate { 
            combatant_id: agent.combatant().id,
            position: new_combatant_position
        });

        Some(events)
    }
}