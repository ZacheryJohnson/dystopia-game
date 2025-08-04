use std::sync::{Arc, Mutex};
use dys_satisfiable::SatisfiableField;
use dys_world::attribute::attribute_type::AttributeType;
use dys_world::combatant::instance::CombatantInstanceId;
use crate::ai::agent::Agent;
use crate::ai::belief::SatisfiableBelief;
use crate::ai::beliefs::belief_set::BeliefSet;
use crate::ai::strategy::Strategy;
use crate::game_state::GameState;
use crate::simulation::simulation_event::{PendingSimulationEvent, SimulationEvent};

const SHOVE_FORCE_MULTIPLIER: f32 = 15000.0;

pub struct ShoveCombatantStrategy {
    self_combatant_id: CombatantInstanceId,
    target_combatant_id: CombatantInstanceId,
    is_complete: bool,
}

impl ShoveCombatantStrategy {
    pub fn new(self_combatant_id: CombatantInstanceId, target_combatant_id: CombatantInstanceId) -> ShoveCombatantStrategy {
        ShoveCombatantStrategy {
            self_combatant_id,
            target_combatant_id,
            is_complete: false,
        }
    }
}

impl Strategy for ShoveCombatantStrategy {
    fn name(&self) -> String {
        String::from("Shove Combatant")
    }

    fn can_perform(&self, beliefs: &BeliefSet) -> bool {
        beliefs.can_satisfy(
            &SatisfiableBelief::CanReachCombatant()
                .self_combatant_id(SatisfiableField::Exactly(self.self_combatant_id))
                .target_combatant_id(SatisfiableField::Exactly(self.target_combatant_id))
        )
    }

    fn should_interrupt(&self, _: &BeliefSet) -> bool {
        false
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    #[tracing::instrument(
        fields(combatant_id = agent.combatant().id),
        skip_all,
        level = "trace"
    )]
    fn tick(
        &mut self,
        agent: &dyn Agent,
        game_state: Arc<Mutex<GameState>>,
    ) -> Option<Vec<PendingSimulationEvent>> {
        let mut events = vec![];

        let mut game_state = game_state.lock().unwrap();
        let combatants = game_state.combatants.clone();
        let (rigid_body_set, _) = game_state.physics_sim.sets_mut();
        let self_object = combatants.get(&self.self_combatant_id).unwrap();
        let target_object = combatants.get(&self.target_combatant_id).unwrap();

        let self_pos = rigid_body_set.get(self_object.rigid_body_handle).unwrap().translation();
        let target_pos = rigid_body_set.get(target_object.rigid_body_handle).unwrap().translation();

        let force_direction = (target_pos - self_pos).normalize();
        let force_magnitude = {
            let combatant_instance = agent.combatant().combatant.lock().unwrap();
            let strength = combatant_instance
                .get_attribute_value(&AttributeType::Strength)
                .unwrap_or_default()
                * SHOVE_FORCE_MULTIPLIER;

            let target_weight = target_object.weight();

            strength / target_weight
        };

        events.push(PendingSimulationEvent(
            SimulationEvent::CombatantShoveForceApplied {
                shover_combatant_id: self.self_combatant_id,
                recipient_target_id: self.target_combatant_id,
                force_magnitude,
                force_direction,
            }
        ));

        // ZJ-TODO: this should be handled elsewhere
        events.push(PendingSimulationEvent(
            SimulationEvent::CombatantStunned {
                combatant_id: self.target_combatant_id,
                start: true
            }
        ));

        Some(events)
    }
}
