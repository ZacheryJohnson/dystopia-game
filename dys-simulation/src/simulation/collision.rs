use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::game_objects::ball::BallState;
use crate::game_objects::game_object_type::GameObjectType;
use crate::game_state::GameState;
use crate::simulation::simulation_event::{PendingSimulationEvent, SimulationEvent};
use crate::simulation::simulation_stage::SimulationStage;

pub(crate) fn handle_collision_events(game_state: Arc<Mutex<GameState>>) -> SimulationStage {
    let start_time = Instant::now();
    let mut new_simulation_events = vec![];

    let (collision_events, active_colliders, balls) = {
        let mut game_state = game_state.lock().unwrap();

        let collision_events = game_state.physics_sim.collision_events().to_owned();
        let active_colliders = game_state.active_colliders.to_owned();
        let balls = game_state.balls.to_owned();

        (collision_events, active_colliders, balls)
    };

    while let Ok(evt) = collision_events.try_recv() {
        let maybe_collider_1 = active_colliders.get(&evt.collider1());
        let maybe_collider_2 = active_colliders.get(&evt.collider2());
        if maybe_collider_1.is_none() || maybe_collider_2.is_none() {
            tracing::warn!("why tho");
            continue;
        }

        let collider_1 = maybe_collider_1.unwrap();
        let collider_2 = maybe_collider_2.unwrap();

        match (collider_1, collider_2) {
            (GameObjectType::Invalid, _) | (_, GameObjectType::Invalid) => continue,
            (GameObjectType::Ball(ball_id), GameObjectType::Barrier) | (GameObjectType::Barrier, GameObjectType::Ball(ball_id)) => {
                // We don't care if the ball and barrier have stopped colliding
                if evt.stopped() {
                    continue;
                }

                let ball_obj = balls.get(ball_id).expect("Received invalid ball ID");

                if let BallState::ThrownAtTarget { direction: _direction, thrower_id, target_id } = ball_obj.state {
                    // ZJ-TODO: do this in simulation: Ball aiming for a target hit the arena - mark it as rolling now
                    new_simulation_events.push(PendingSimulationEvent(
                        SimulationEvent::BallCollisionArena { thrower_id, original_target_id: target_id, ball_id: *ball_id }
                    ));
                }
            },
            (GameObjectType::Plate(plate_id), GameObjectType::Combatant(combatant_id)) | (GameObjectType::Combatant(combatant_id), GameObjectType::Plate(plate_id)) => {
                if evt.started() {
                    new_simulation_events.push(PendingSimulationEvent(
                        SimulationEvent::CombatantOnPlate { combatant_id: *combatant_id, plate_id: *plate_id }
                    ));
                }

                if evt.stopped() {
                    new_simulation_events.push(PendingSimulationEvent(
                        SimulationEvent::CombatantOffPlate { combatant_id: *combatant_id, plate_id: *plate_id }
                    ));
                }
            },
            (GameObjectType::Plate(_), _) | (_, GameObjectType::Plate(_)) => continue,
            (GameObjectType::BallSpawn, _) | (_, GameObjectType::BallSpawn) => continue,
            (GameObjectType::Barrier, GameObjectType::Combatant(combatant_id)) | (GameObjectType::Combatant(combatant_id), GameObjectType::Barrier) => {
                if !evt.started() {
                    continue;
                }

                let damage = {
                    let game_state = game_state.lock().unwrap();

                    let combatant_rb = game_state.combatants.get(combatant_id).unwrap().rigid_body_handle;

                    let (rigid_body_set, _) = game_state.physics_sim.sets();
                    let combatant_rigid_body = rigid_body_set.get(combatant_rb).unwrap();
                    combatant_rigid_body.linvel().magnitude()
                };

                let mut game_state = game_state.lock().unwrap();
                let combatant_object = game_state.combatants.get_mut(combatant_id).unwrap();
                combatant_object.apply_damage(damage);
            },
            (GameObjectType::Barrier, _) => continue,
            (GameObjectType::Ball(_), GameObjectType::Ball(_)) => continue,
            (GameObjectType::Combatant(_), GameObjectType::Combatant(_)) => continue,
            (GameObjectType::Ball(ball_id), GameObjectType::Combatant(combatant_id)) | (GameObjectType::Combatant(combatant_id), GameObjectType::Ball(ball_id)) => {
                // We don't care if the ball and combatant have stopped colliding
                if evt.stopped() {
                    continue;
                }

                let ball_obj = balls.get(ball_id).expect("Received invalid ball ID");

                if let BallState::ThrownAtTarget { direction: _, thrower_id, target_id: _ } = ball_obj.state {
                    let game_state = game_state.lock().unwrap();
                    let thrower_team = game_state.combatants.get(&thrower_id).unwrap().team;
                    let hit_combatant_team = game_state.combatants.get(combatant_id).unwrap().team;

                    if thrower_team != hit_combatant_team {
                        new_simulation_events.push(PendingSimulationEvent(
                            SimulationEvent::BallCollisionEnemy { thrower_id, enemy_id: *combatant_id, ball_id: *ball_id }
                        ));
                    }

                    // ZJ-TODO: need to handle case of same team (catch pass?)
                }
            }
        }
    }

    SimulationStage {
        execution_duration: start_time.elapsed(),
        pending_events: new_simulation_events
    }
}