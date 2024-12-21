use crate::{ai::{agent::Agent, belief::Belief, strategy::Strategy}, game_objects::game_object::GameObject, game_state::GameState, simulation::simulation_event::SimulationEvent};

pub(in crate::ai) struct NoopStrategy;

impl Strategy for NoopStrategy {
    fn name(&self) -> String {
        String::from("Noop")
    }

    fn can_perform(&self, _: &[Belief]) -> bool {
        true
    }

    fn is_complete(&self) -> bool {
        true
    }

    fn tick(
        &mut self,
        agent: &mut dyn Agent,
        game_state: &mut GameState,
    ) -> Option<Vec<SimulationEvent>> {
        let (rigid_body_set, _, _) = game_state.physics_sim.sets();
        let combatant_rb_handle = agent.combatant().rigid_body_handle().unwrap();
        let combatant_rb = rigid_body_set.get(combatant_rb_handle).unwrap();

        Some(vec![
            SimulationEvent::CombatantPositionUpdate {
                combatant_id: agent.combatant().id,
                position: *combatant_rb.translation(),
            }
        ])
    }
}