use std::sync::{Arc, Mutex};
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
        // ZJ-TODO: out earlier if we know the ball has already been picked up by someone else
        //          stretch goal - if we believe we have no chance of getting to the ball first, abort early?
        !owned_beliefs.contains(&Belief::SelfHasBall) && owned_beliefs.contains(&Belief::SelfCanReachBall { ball_id: self.ball_id })
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn tick(
        &mut self,
        agent: &dyn Agent,
        game_state: Arc<Mutex<GameState>>,
    ) -> Option<Vec<SimulationEvent>> {
        let balls = {
            let game_state = game_state.lock().unwrap();
            game_state
                .balls
                .clone()
        };

        let Some(ball_object) = balls.get(&self.ball_id) else {
            tracing::error!("Failed to find ball object {}", self.ball_id);
            self.is_complete = true;
            return None;
        };

        if let Some(holder_combatant_id) = ball_object.held_by {
            tracing::debug!("Failed to pick up ball object; currently held by combatant {}", holder_combatant_id);
            self.is_complete = true;
            return None;
        }

        Some(vec![
            SimulationEvent::CombatantPickedUpBall { combatant_id: agent.combatant().id, ball_id: self.ball_id }
        ])
    }
}
