use crate::{ai::{agent::Agent, belief::Belief, strategy::Strategy}, game_objects::ball::BallId, game_state::GameState, simulation::simulation_event::SimulationEvent};

pub struct PickUpBallStrategy {
    ball_id: BallId,
    is_complete: bool,
}

impl PickUpBallStrategy {
    pub fn new(target_ball: BallId) -> PickUpBallStrategy {
        PickUpBallStrategy {
            ball_id: target_ball,
            is_complete: false,
        }
    }
}

impl Strategy for PickUpBallStrategy {
    fn name(&self) -> String {
        format!("Pick Up Ball {}", self.ball_id)
    }

    fn can_perform(&self, owned_beliefs: &[Belief]) -> bool {
        owned_beliefs.contains(&Belief::BallNotHeld { ball_id: self.ball_id }) && owned_beliefs.contains(&Belief::SelfCanReachBall { ball_id: self.ball_id })
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn tick(
        &mut self,
        agent: &mut dyn Agent,
        game_state: &mut GameState
    ) -> Option<Vec<SimulationEvent>> {
        let Some(ball_object) = game_state.balls.get_mut(&self.ball_id) else {
            tracing::error!("Failed to find ball object {}", self.ball_id);
            self.is_complete = true;
            return None;
        };

        if let Some(holder_combatant_id) = ball_object.held_by {
            tracing::debug!("Failed to pick up ball object; currently held by combatant {}", holder_combatant_id);
            self.is_complete = true;
            return None;
        }

        ball_object.set_held_by(Some(agent.combatant().id), game_state.current_tick);

        Some(vec![
            SimulationEvent::CombatantPickedUpBall { combatant_id: agent.combatant().id, ball_id: self.ball_id }
        ])
    }
}