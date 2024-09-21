use rapier3d::{na::Point3, prelude::RigidBodyHandle};

use crate::{ai::{agent::Agent, strategy::Strategy}, game_state::GameState, simulation::simulation_event::SimulationEvent};

pub struct MoveToLocationStrategy {
    can_perform: bool,
    is_complete: bool,
    target_location: Point3<f32>,
    combatant_rigid_body_handle: RigidBodyHandle,
}

impl MoveToLocationStrategy {
    pub fn new(target_location: Point3<f32>, combatant_rigid_body_handle: RigidBodyHandle) -> MoveToLocationStrategy {
        MoveToLocationStrategy {
            can_perform: true,
            is_complete: false,
            target_location,
            combatant_rigid_body_handle,
        }
    }
}

impl Strategy for MoveToLocationStrategy {
    fn can_perform(&self) -> bool {
        self.can_perform
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn tick(
        &mut self,
        agent: &mut dyn Agent,
        game_state: &mut GameState,
    ) -> Vec<SimulationEvent> {
        let mut events = vec![];

        if !self.can_perform {
            tracing::warn!("Trying to tick strategy but can_perform is false!");
            return events;
        }

        let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();
        let combatant_rb = rigid_body_set.get_mut(self.combatant_rigid_body_handle).unwrap();
        let combatant_position = combatant_rb.translation();

        // ZJ-TODO: HACK: y coordinate is wonky
        //                ignore whatever we see initially and just maintain the combatant's y-pos
        self.target_location.y = combatant_position.y;

        // ZJ-TODO: read this from combatant stats
        let mut total_distance_can_travel_this_tick = 2.0_f32;
        let mut new_combatant_position = combatant_position.to_owned();

        let arena_navmesh = &game_state.arena_navmesh;
        let unit_resolution = arena_navmesh.config().unit_resolution;
        while total_distance_can_travel_this_tick >= unit_resolution {
            let Some(next_point) = arena_navmesh.get_next_point(new_combatant_position.into(), self.target_location) else {
                break;
            };
    
            let lerp_distance = (total_distance_can_travel_this_tick - unit_resolution).clamp(0.0, unit_resolution);
            new_combatant_position = new_combatant_position.lerp(&next_point.coords, lerp_distance);
            total_distance_can_travel_this_tick = (total_distance_can_travel_this_tick - unit_resolution).max(0.0);
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

        events
    }
}