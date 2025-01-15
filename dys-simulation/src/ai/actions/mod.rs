use std::sync::{Arc, Mutex};
use dys_satisfiable::SatisfiableField;
use crate::{ai::{action::ActionBuilder, belief::Belief, strategies::move_to_location::MoveToLocationStrategy}, game_objects::{combatant::CombatantObject, game_object::GameObject}, game_state::GameState};
use crate::ai::belief::SatisfiableBelief;
use super::{action::Action, strategies::{pick_up_ball::PickUpBallStrategy, throw_ball_at_target_location::ThrowBallAtTargetStrategy}};

/// ZJ-TODO: HACK: this value should be passed in through simulation settings.
/// This value allows us to make all movement actions cheaper/more expensive,
/// as other actions may have lower/higher absolute costs.
const MOVE_TO_LOCATION_WEIGHT_HARDCODE_HACK: f32 = 0.2_f32;
const MOVE_TO_BALL_WEIGHT_HARDCODE_HACK: f32 = 0.2_f32;

#[tracing::instrument(fields(combatant_id = combatant.id), skip_all, level = "trace")]
pub fn actions(
    combatant: &CombatantObject,
    game_state: Arc<Mutex<GameState>>,
) -> Vec<Action> {
    let mut actions = vec![];

    let combatant_pos = {
        let game_state = game_state.lock().unwrap();
        let (rigid_body_set, _, _) = game_state.physics_sim.sets();
        rigid_body_set
            .get(combatant.rigid_body_handle)
            .unwrap()
            .translation()
            .to_owned()
    };

    let combatant_move_speed = combatant.combatant.lock().unwrap().move_speed();

    let (plates, balls, combatants) = {
        let game_state = game_state.lock().unwrap();
        (game_state.plates.clone(), game_state.balls.clone(), game_state.combatants.clone())
    };

    for (plate_id, plate_object) in plates {
        let plate_location = {
            let game_state = game_state.lock().unwrap();
            let (_, collider_set, _) = game_state.physics_sim.sets();
            collider_set
                .get(plate_object.collider_handle().unwrap())
                .unwrap()
                .translation()
                .to_owned()
        };

        actions.push(
            ActionBuilder::new()
                .name(format!("Move to Plate {plate_id}"))
                .strategy(MoveToLocationStrategy::new(
                    combatant_pos.into(),
                    plate_location.into(),
                    game_state.clone())
                )
                .cost(MOVE_TO_LOCATION_WEIGHT_HARDCODE_HACK * (plate_location - combatant_pos).magnitude() / combatant_move_speed)
                .promised(vec![Belief::OnPlate { combatant_id: combatant.id, plate_id }])
                .build()
        );
    }

    for (ball_id, ball_object) in balls {
        if ball_object.held_by.is_some() {
            continue;
        }

        let ball_location = {
            let game_state = game_state.lock().unwrap();
            let (rigid_body_set, _, _) = game_state.physics_sim.sets();
            rigid_body_set
                .get(ball_object.rigid_body_handle().unwrap())
                .unwrap()
                .translation()
                .to_owned()
        };

        actions.push(
            ActionBuilder::new()
                .name(format!("Move to Ball {ball_id}"))
                .strategy(MoveToLocationStrategy::new(
                    combatant_pos.into(),
                    ball_location.into(),
                    game_state.clone())
                )
                .cost(MOVE_TO_BALL_WEIGHT_HARDCODE_HACK * (ball_location - combatant_pos).magnitude() / combatant_move_speed) 
                .prohibited(vec![
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(combatant.id))
                ])
                .completion(vec![
                    Belief::InBallPickupRange { ball_id, combatant_id: combatant.id },
                ])
                .build()
        );

        actions.push(
            ActionBuilder::new()
                .name(format!("Pick Up Ball {ball_id}"))
                .strategy(PickUpBallStrategy::new(combatant.id, combatant_pos, ball_id))
                .prerequisites(vec![
                    SatisfiableBelief::InBallPickupRange()
                        .combatant_id(SatisfiableField::Exactly(combatant.id))
                        .ball_id(SatisfiableField::Exactly(ball_id))
                ])
                .completion(vec![
                    Belief::HeldBall { ball_id, combatant_id: combatant.id },
                ])
                .build()
        );
    }

    for (target_combatant_id, _) in combatants {
        // Don't try to throw a ball at ourselves
        if target_combatant_id == combatant.id {
            continue;
        }

        actions.push(
            ActionBuilder::new()
                .name(format!("Throw Ball at/to Combatant {}", target_combatant_id))
                .strategy(ThrowBallAtTargetStrategy::new(combatant.id, target_combatant_id))
                .cost(10.0_f32 /* ZJ-TODO */)
                .prerequisites(vec![
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(combatant.id))
                ])
                .build()
        );
    }

    actions
}
