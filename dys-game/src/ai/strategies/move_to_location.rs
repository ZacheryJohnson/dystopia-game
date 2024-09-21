use rapier3d::{na::Point3, prelude::RigidBodyHandle};

use crate::{ai::{agent::Agent, strategy::Strategy}, game_objects::combatant::CombatantState, game_state::GameState};

pub struct MoveToLocationStrategy {
    can_perform: bool,
    is_complete: bool,
    target_location: Point3<f32>,
    combatant_rigid_body_handle: RigidBodyHandle,
}

impl MoveToLocationStrategy {
    pub fn new(target_location: Point3<f32>, combatant_rigid_body_handle: RigidBodyHandle) -> MoveToLocationStrategy {
        MoveToLocationStrategy {
            can_perform: false,
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

    fn start(&mut self, agent: &mut dyn Agent, _game_state: &mut GameState) {
        // no-op
    }

    fn tick(
        &mut self,
        agent: &mut dyn Agent,
        game_state: &mut GameState,
    ) {
        if !self.can_perform {
            tracing::warn!("Trying to tick strategy but can_perform is false!");
            return;
        }

        let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();
        let combatant_rb = rigid_body_set.get_mut(self.combatant_rigid_body_handle).unwrap();
        let combatant_position = combatant_rb.translation();

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
    
        // ZJ-TODO: don't blindly copy original y
        //          this assumes we're on a perfectly flat plane
        new_combatant_position.y = combatant_position.y;
        combatant_rb.set_translation(new_combatant_position, true);
        //combatant_rb.set_next_kinematic_translation(new_combatant_position);
    }

    fn stop(&mut self, agent: &mut dyn Agent, _game_state: &mut GameState) {
        // no-op
    }
}