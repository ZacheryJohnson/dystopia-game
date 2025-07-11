use std::rc::Rc;
use std::sync::{Arc, Mutex};
use rapier3d::{na::Vector3};
use dys_satisfiable::SatisfiableField;
use crate::{ai::{agent::Agent, strategy::Strategy}, game_objects::ball::BallId, game_state::GameState, simulation::simulation_event::SimulationEvent};
use crate::ai::belief::SatisfiableBelief;
use crate::ai::beliefs::belief_set::BeliefSet;
use crate::game_objects::combatant::CombatantId;
use crate::simulation::simulation_event::PendingSimulationEvent;

pub struct PickUpBallStrategy {
    self_combatant_id: CombatantId,
    ball_id: BallId,
    believed_ball_position: Vector3<f32>,
    is_complete: bool,
}

impl PickUpBallStrategy {
    pub fn new(
        self_id: CombatantId,
        target_ball: BallId,
        believed_ball_position: Vector3<f32>
    ) -> PickUpBallStrategy {
        PickUpBallStrategy {
            self_combatant_id: self_id,
            ball_id: target_ball,
            believed_ball_position,
            is_complete: false,
        }
    }
}

impl Strategy for PickUpBallStrategy {
    fn name(&self) -> String {
        format!("Pick Up Ball {}", self.ball_id)
    }

    #[tracing::instrument(name = "strategy::pick_up_ball::can_perform", skip_all, level = "trace")]
    fn can_perform(&self, owned_beliefs: &BeliefSet) -> bool {
        let self_not_holding_any_ball = owned_beliefs.all_satisfy(
            SatisfiableBelief::HeldBall()
                .combatant_id(SatisfiableField::NotExactly(self.self_combatant_id))
        );

        let self_can_reach_ball = owned_beliefs.can_satisfy(
            &SatisfiableBelief::InBallPickupRange()
                .ball_id(SatisfiableField::Exactly(self.ball_id))
                .combatant_id(SatisfiableField::Exactly(self.self_combatant_id))
        );

        tracing::debug!("Not holding ball={self_not_holding_any_ball}");
        tracing::debug!("Can reach ball={self_can_reach_ball}");

        let can_perform = !self.should_interrupt(owned_beliefs) && self_not_holding_any_ball && self_can_reach_ball;
        tracing::debug!("Can perform: {can_perform}");
        can_perform
    }

    #[tracing::instrument(name = "strategy::pick_up_ball::should_interrupt", skip_all, level = "trace")]
    fn should_interrupt(&self, owned_beliefs: &BeliefSet) -> bool {
        // If someone picks up the ball we're targeting, interrupt
        let other_combatant_holding_target_ball = owned_beliefs.can_satisfy(
            &SatisfiableBelief::HeldBall()
                .ball_id(SatisfiableField::Exactly(self.ball_id))
        );

        // If we don't know where the ball we're targeting is, interrupt
        let target_ball_now_unknown = !owned_beliefs.can_satisfy(
            &SatisfiableBelief::BallPosition()
                .ball_id(SatisfiableField::Exactly(self.ball_id))
        );

        // If the ball has moved significantly from where we initially planned, interrupt
        let believed_ball_pos = self.believed_ball_position.to_owned();
        let target_ball_moved_significantly = !owned_beliefs.can_satisfy(
            &SatisfiableBelief::BallPosition()
                .ball_id(SatisfiableField::Exactly(self.ball_id))
                .position(SatisfiableField::Lambda(Rc::new(move |pos| {
                    (believed_ball_pos - pos).magnitude() <= 1.0
                })))
        );

        tracing::debug!("Other holding target ball={other_combatant_holding_target_ball}");
        tracing::debug!("Target ball unknown={target_ball_now_unknown}");
        tracing::debug!("Target ball moved significantly={target_ball_moved_significantly}");

        let should_interrupt = other_combatant_holding_target_ball || target_ball_now_unknown || target_ball_moved_significantly;
        tracing::debug!("Should interrupt: {should_interrupt}");
        should_interrupt
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn tick(
        &mut self,
        agent: &dyn Agent,
        game_state: Arc<Mutex<GameState>>,
    ) -> Option<Vec<PendingSimulationEvent>> {
        let balls = {
            let game_state = game_state.lock().unwrap();
            game_state
                .balls
                .clone()
        };

        let Some(ball_object) = balls.get(&self.ball_id) else {
            tracing::error!("Failed to find ball object {}", self.ball_id);
            return None;
        };

        if let Some(holder_combatant_id) = ball_object.held_by {
            tracing::debug!("Failed to pick up ball object; currently held by combatant {}", holder_combatant_id);
            return None;
        }

        self.is_complete = true;

        Some(vec![
            PendingSimulationEvent(
                SimulationEvent::CombatantPickedUpBall {
                    combatant_id: agent.combatant().id,
                    ball_id: self.ball_id
                }
            )
        ])
    }
}
