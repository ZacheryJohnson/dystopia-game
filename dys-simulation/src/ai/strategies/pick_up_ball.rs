use std::sync::{Arc, Mutex};
use rapier3d::{na::Vector3};
use dys_satisfiable::SatisfiableField;
use crate::{ai::{agent::Agent, strategy::Strategy}, game_objects::ball::BallId, game_state::GameState, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::{BeliefSet, SatisfiableBelief};
use crate::game_objects::combatant::CombatantId;

pub struct PickUpBallStrategy {
    self_combatant_id: CombatantId,
    self_combatant_position: Vector3<f32>,
    self_combatant_reach: f32,
    ball_id: BallId,
    is_complete: bool,
}

impl PickUpBallStrategy {
    pub fn new(self_id: CombatantId, self_position: Vector3<f32>, target_ball: BallId) -> PickUpBallStrategy {
        PickUpBallStrategy {
            self_combatant_id: self_id,
            self_combatant_position: self_position,
            self_combatant_reach: 1.0, // ZJ-TODO: get this from stats
            ball_id: target_ball,
            is_complete: false,
        }
    }
}

impl Strategy for PickUpBallStrategy {
    fn name(&self) -> String {
        format!("Pick Up Ball {}", self.ball_id)
    }

    fn can_perform(&self, owned_beliefs: &BeliefSet) -> bool {
        // ZJ-TODO: out earlier if we know the ball has already been picked up by someone else
        //          stretch goal - if we believe we have no chance of getting to the ball first, abort early?
        let self_not_holding_ball = owned_beliefs.can_satisfy(
            SatisfiableBelief::HeldBall()
                .combatant_id(SatisfiableField::NotExactly(self.self_combatant_id))
        );

        let self_can_reach_ball = owned_beliefs.can_satisfy(
            SatisfiableBelief::BallPosition()
                .ball_id(SatisfiableField::Exactly(self.ball_id))
                // .position(SatisfiableField::) ZJ-TODO: implement ::Within or similar
        );

        self_not_holding_ball && self_can_reach_ball
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
