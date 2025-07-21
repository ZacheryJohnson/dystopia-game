use std::sync::{Arc, Mutex};
use indexmap::IndexMap;
use rapier3d::na::Vector3;
use dys_satisfiable::SatisfiableField;
use crate::{ai::{action::ActionBuilder, belief::Belief, strategies::move_to_location::MoveToLocationStrategy}, game_objects::{combatant::CombatantObject, game_object::GameObject}, game_state::GameState};
use crate::ai::belief::SatisfiableBelief;
use crate::ai::strategies::shove_combatant::ShoveCombatantStrategy;
use crate::game_objects::game_object_type::GameObjectType;
use super::{action::Action, strategies::{pick_up_ball::PickUpBallStrategy, throw_ball_at_target_location::ThrowBallAtTargetStrategy}};

/// ZJ-TODO: HACK: this value should be passed in through simulation settings.
/// This value allows us to make all movement actions cheaper/more expensive,
/// as other actions may have lower/higher absolute costs.
const MOVE_TO_LOCATION_WEIGHT_HARDCODE_HACK: f32 = 0.8_f32;
const MOVE_TO_BALL_WEIGHT_HARDCODE_HACK: f32 = 0.4_f32;

#[tracing::instrument(fields(combatant_id = combatant.id), skip_all, level = "trace")]
pub fn actions(
    combatant: &CombatantObject,
    game_state: Arc<Mutex<GameState>>,
) -> Vec<Action> {
    let mut actions = vec![];
    let current_tick = game_state.lock().unwrap().current_tick.to_owned();

    let combatant_pos = {
        let game_state = game_state.lock().unwrap();
        let (rigid_body_set, _) = game_state.physics_sim.sets();
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
            let (_, collider_set) = game_state.physics_sim.sets();
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
                    combatant.id,
                    plate_location.into(),
                    4)
                )
                .cost(MOVE_TO_LOCATION_WEIGHT_HARDCODE_HACK * (plate_location - combatant_pos).magnitude() / combatant_move_speed)
                .promises(Belief::OnPlate { combatant_id: combatant.id, plate_id })
                .build()
        );
    }

    for (other_combatant_id, other_combatant_object) in &combatants {
        // Don't add actions that refer to ourselves
        if combatant.id == *other_combatant_id {
            continue;
        }

        let target_pos = {
            let game_state = game_state.lock().unwrap();
            let (rigid_body_set, _) = game_state.physics_sim.sets();
            rigid_body_set
                .get(other_combatant_object.rigid_body_handle)
                .unwrap()
                .translation()
                .to_owned()
        };

        actions.push(
            ActionBuilder::new()
                .name(format!("Look For Combatant {other_combatant_id}"))
                .strategy(MoveToLocationStrategy::new_with_target_object(
                    combatant.id,
                    GameObjectType::Combatant(*other_combatant_id),
                    4)
                )
                .cost(MOVE_TO_LOCATION_WEIGHT_HARDCODE_HACK * (target_pos - combatant_pos).magnitude() / combatant_move_speed)
                .completion(vec![
                    Belief::ScannedEnvironment { tick: current_tick },
                ])
                .promises(Belief::DirectLineOfSightToCombatant {
                    self_combatant_id: combatant.id,
                    other_combatant_id: *other_combatant_id,
                })
                .consumes(SatisfiableBelief::ScannedEnvironment())
                .build()
        );

        actions.push(
            ActionBuilder::new()
                .name(format!("Move to Combatant {other_combatant_id}"))
                .strategy(MoveToLocationStrategy::new_with_target_tracking(
                    combatant.id,
                    GameObjectType::Combatant(*other_combatant_id),
                ))
                .cost(MOVE_TO_LOCATION_WEIGHT_HARDCODE_HACK * (target_pos - combatant_pos).magnitude() / combatant_move_speed)
                .promises(Belief::CanReachCombatant {
                    self_combatant_id: combatant.id,
                    target_combatant_id: *other_combatant_id,
                })
                .build()
        );

        actions.push(
            ActionBuilder::new()
                .name(format!("Shove Combatant {other_combatant_id}"))
                .strategy(ShoveCombatantStrategy::new(
                    combatant.id,
                    *other_combatant_id
                ))
                .cost(15.0) // ZJ-TODO
                .requires(
                    SatisfiableBelief::CanReachCombatant()
                        .self_combatant_id(SatisfiableField::Exactly(combatant.id))
                        .target_combatant_id(SatisfiableField::Exactly(*other_combatant_id)),
                )
                .promises(Belief::CombatantShoved {
                    combatant_id: *other_combatant_id,
                    on_plate: other_combatant_object.plate(),
                })
                .consumes(SatisfiableBelief::CombatantShoved()
                    .combatant_id(SatisfiableField::Exactly(*other_combatant_id)),
                )
                .build()
        )
    }

    for (ball_id, ball_object) in balls {
        let ball_location = {
            let game_state = game_state.lock().unwrap();
            let (rigid_body_set, _) = game_state.physics_sim.sets();
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
                    combatant.id,
                    ball_location.into(),
                    4)
                )
                .cost(MOVE_TO_BALL_WEIGHT_HARDCODE_HACK * (ball_location - combatant_pos).magnitude() / combatant_move_speed)
                .requires(
                    SatisfiableBelief::BallPosition()
                        .ball_id(SatisfiableField::Exactly(ball_id))
                )
                .prohibits(
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(combatant.id))
                )
                .promises(Belief::InBallPickupRange { ball_id, combatant_id: combatant.id })
                .build()
        );

        actions.push(
            ActionBuilder::new()
                .name(format!("Pick Up Ball {ball_id}"))
                .strategy(PickUpBallStrategy::new(combatant.id, ball_id, ball_location.to_owned()))
                .cost(1.0)
                .requires(
                    SatisfiableBelief::InBallPickupRange()
                        .combatant_id(SatisfiableField::Exactly(combatant.id))
                        .ball_id(SatisfiableField::Exactly(ball_id))
                )
                .prohibits(
                    SatisfiableBelief::HeldBall()
                        .combatant_id(SatisfiableField::Exactly(combatant.id))
                )
                .prohibits(
                    SatisfiableBelief::HeldBall()
                        .ball_id(SatisfiableField::Exactly(ball_id))
                )
                .prohibits(
                    SatisfiableBelief::BallIsFlying()
                        .ball_id(SatisfiableField::Exactly(ball_id))
                )
                .completion(vec![
                    Belief::HeldBall { ball_id, combatant_id: combatant.id },
                ])
                .build()
        );

        let enemy_combatant_ids = combatants
            .clone()
            .iter()
            .filter(|(_, combatant_object)| combatant_object.team != combatant.team)
            .map(|(combatant_id, _)| combatant_id.to_owned())
            .collect::<Vec<_>>();

        let teammate_combatant_ids = combatants
            .clone()
            .iter()
            .filter(|(_, combatant_object)| combatant_object.team == combatant.team)
            .map(|(combatant_id, _)| combatant_id.to_owned())
            .collect::<Vec<_>>();

        for (target_combatant_id, _) in combatants.clone() {
            // Don't try to throw a ball at ourselves
            if target_combatant_id == combatant.id {
                continue;
            }

            actions.push(
                ActionBuilder::new()
                    .name(format!("Catch Ball {ball_id}"))
                    .strategy(PickUpBallStrategy::new(combatant.id, ball_id, ball_location.to_owned()))
                    .cost(1.0) // ZJ-TODO
                    .requires(
                        SatisfiableBelief::InBallPickupRange()
                            .combatant_id(SatisfiableField::Exactly(combatant.id))
                            .ball_id(SatisfiableField::Exactly(ball_id))
                    )
                    .requires(
                        SatisfiableBelief::BallThrownAtCombatant()
                            .target_combatant_id(SatisfiableField::Exactly(combatant.id))
                            .ball_id(SatisfiableField::Exactly(ball_id))
                    )
                    .prohibits(
                        SatisfiableBelief::HeldBall()
                            .combatant_id(SatisfiableField::Exactly(combatant.id))
                    )
                    .prohibits(
                        SatisfiableBelief::HeldBall()
                            .ball_id(SatisfiableField::Exactly(ball_id))
                    )
                    .requires(
                        SatisfiableBelief::BallIsFlying()
                            .ball_id(SatisfiableField::Exactly(ball_id))
                    )
                    .completion(vec![
                        Belief::HeldBall { ball_id, combatant_id: combatant.id },
                        Belief::BallCaught { ball_id, combatant_id: combatant.id, thrower_id: target_combatant_id },
                    ])
                    .consumes(
                        SatisfiableBelief::BallIsFlying()
                            .ball_id(SatisfiableField::Exactly(ball_id))
                    )
                    .consumes(
                        SatisfiableBelief::BallCaught()
                            .ball_id(SatisfiableField::Exactly(ball_id))
                            .combatant_id(SatisfiableField::Exactly(combatant.id))
                    )
                    .build()
            );
        }

        let teammate_combatants = combatants
            .clone()
            .iter()
            .filter(|(_, combatant_object)| combatant_object.team == combatant.team)
            .map(|(id, obj)| (id.to_owned(), obj.to_owned()))
            .collect::<IndexMap<_, _>>();

        for (teammate_combatant_id, teammate_combatant_object) in teammate_combatants {
            let target_pos = {
                let game_state = game_state.lock().unwrap();
                let (rigid_body_set, _) = game_state.physics_sim.sets();
                rigid_body_set
                    .get(teammate_combatant_object.rigid_body_handle)
                    .unwrap()
                    .translation()
                    .to_owned()
            };

            actions.push(
                ActionBuilder::new()
                    .name(format!("Pass Ball {ball_id} to Combatant {teammate_combatant_id}"))
                    .strategy(ThrowBallAtTargetStrategy::new(combatant.id, teammate_combatant_id))
                    // ZJ-TODO: ideally this is an inverse bell curve
                    //          for now, just penalize close throws and reward far throws
                    .cost(10.0 + 5.0 / (target_pos - combatant_pos).magnitude())
                    .requires(
                        SatisfiableBelief::HeldBall()
                            .combatant_id(SatisfiableField::Exactly(combatant.id))
                            .ball_id(SatisfiableField::Exactly(ball_id))
                    )
                    .requires(
                        SatisfiableBelief::DirectLineOfSightToCombatant()
                            .self_combatant_id(SatisfiableField::Exactly(combatant.id))
                            .other_combatant_id(SatisfiableField::Exactly(teammate_combatant_id))
                    )
                    .requires(
                        SatisfiableBelief::DirectLineOfSightToCombatant()
                            .other_combatant_id(SatisfiableField::In(teammate_combatant_ids.clone()))
                    )
                    .prohibits(
                        SatisfiableBelief::CombatantPosition()
                            .combatant_id(SatisfiableField::In(enemy_combatant_ids.clone()))
                            .position(SatisfiableField::lambda_from(move |target_pos: Vector3<f32>| {
                                const MIN_THROW_DISTANCE: f32 = 5.0;
                                (target_pos - combatant_pos).magnitude() < MIN_THROW_DISTANCE
                            }))
                    )
                    .prohibits(
                        SatisfiableBelief::CombatantIsStunned()
                            .combatant_id(SatisfiableField::Exactly(teammate_combatant_id))
                    )
                    .requires(
                        SatisfiableBelief::CombatantPosition()
                            .combatant_id(SatisfiableField::Exactly(teammate_combatant_id))
                            .position(SatisfiableField::lambda_from(move |target_pos: Vector3<f32>| {
                                const MIN_THROW_DISTANCE: f32 = 5.0;
                                (target_pos - combatant_pos).magnitude() >= MIN_THROW_DISTANCE
                            }))
                    )
                    .prohibits(
                        SatisfiableBelief::HeldBall()
                            .combatant_id(SatisfiableField::Exactly(teammate_combatant_id))
                    )
                    .completion(vec![
                        Belief::BallThrownAtCombatant {
                            ball_id,
                            thrower_combatant_id: combatant.id,
                            target_combatant_id: teammate_combatant_id,
                            target_on_plate: teammate_combatant_object.plate(),
                        },
                    ])
                    .broadcasts(vec![
                        Belief::BallThrownAtCombatant {
                            ball_id,
                            thrower_combatant_id: combatant.id,
                            target_combatant_id: teammate_combatant_id,
                            target_on_plate: teammate_combatant_object.plate(),
                        }
                    ])
                    .consumes(
                        SatisfiableBelief::HeldBall()
                            .combatant_id(SatisfiableField::Exactly(combatant.id))
                            .ball_id(SatisfiableField::Exactly(ball_id))
                    )
                    .consumes(
                        SatisfiableBelief::BallThrownAtCombatant()
                            .ball_id(SatisfiableField::Exactly(ball_id))
                            .thrower_combatant_id(SatisfiableField::Exactly(combatant.id))
                            .target_combatant_id(SatisfiableField::Exactly(teammate_combatant_id))
                    )
                    .build()
            );
        }


        let enemy_combatants = combatants
            .clone()
            .iter()
            .filter(|(_, combatant_object)| combatant_object.team != combatant.team)
            .map(|(id, obj)| (id.to_owned(), obj.to_owned()))
            .collect::<IndexMap<_, _>>();

        for (enemy_combatant_id, enemy_combatant_object) in enemy_combatants {
            let target_pos = {
                let game_state = game_state.lock().unwrap();
                let (rigid_body_set, _) = game_state.physics_sim.sets();
                rigid_body_set
                    .get(enemy_combatant_object.rigid_body_handle)
                    .unwrap()
                    .translation()
                    .to_owned()
            };

            actions.push(
                ActionBuilder::new()
                    .name(format!("Throw Ball {ball_id} at Combatant {enemy_combatant_id}"))
                    .strategy(ThrowBallAtTargetStrategy::new(combatant.id, enemy_combatant_id))
                    // ZJ-TODO: ideally this is an inverse bell curve
                    //          for now, just penalize close throws and reward far throws
                    .cost(10.0 + 5.0 / (target_pos - combatant_pos).magnitude())
                    .requires(
                        SatisfiableBelief::HeldBall()
                            .combatant_id(SatisfiableField::Exactly(combatant.id))
                            .ball_id(SatisfiableField::Exactly(ball_id))
                    )
                    .requires(
                        SatisfiableBelief::DirectLineOfSightToCombatant()
                            .self_combatant_id(SatisfiableField::Exactly(combatant.id))
                            .other_combatant_id(SatisfiableField::Exactly(enemy_combatant_id))
                    )
                    .requires(
                        SatisfiableBelief::DirectLineOfSightToCombatant()
                            .other_combatant_id(SatisfiableField::In(enemy_combatant_ids.clone()))
                    )
                    .prohibits(
                        SatisfiableBelief::CombatantPosition()
                            .combatant_id(SatisfiableField::In(enemy_combatant_ids.clone()))
                            .position(SatisfiableField::lambda_from(move |target_pos: Vector3<f32>| {
                                const MIN_THROW_DISTANCE: f32 = 5.0;
                                (target_pos - combatant_pos).magnitude() < MIN_THROW_DISTANCE
                            }))
                    )
                    .completion(vec![
                        Belief::BallThrownAtCombatant {
                            ball_id,
                            thrower_combatant_id: combatant.id,
                            target_combatant_id: enemy_combatant_id,
                            target_on_plate: enemy_combatant_object.plate(),
                        },
                    ])
                    .broadcasts(vec![
                        Belief::BallThrownAtCombatant {
                            ball_id,
                            thrower_combatant_id: combatant.id,
                            target_combatant_id: enemy_combatant_id,
                            target_on_plate: enemy_combatant_object.plate(),
                        }
                    ])
                    .consumes(
                        SatisfiableBelief::HeldBall()
                            .combatant_id(SatisfiableField::Exactly(combatant.id))
                            .ball_id(SatisfiableField::Exactly(ball_id))
                    )
                    .consumes(
                        SatisfiableBelief::BallThrownAtCombatant()
                            .ball_id(SatisfiableField::Exactly(ball_id))
                            .thrower_combatant_id(SatisfiableField::Exactly(combatant.id))
                            .target_combatant_id(SatisfiableField::Exactly(enemy_combatant_id))
                    )
                    .build()
            );
        }
    }

    actions
}
