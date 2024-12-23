use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::game_objects::ball::BallState;
use crate::game_objects::game_object_type::GameObjectType;
use crate::game_state::GameState;
use crate::simulation::simulation_event::SimulationEvent;
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

                match ball_obj.state {
                    BallState::ThrownAtTarget { direction: _direction, thrower_id, target_id } => {
                        // ZJ-TODO: do this in simulation: Ball aiming for a target hit the arena - mark it as rolling now
                        new_simulation_events.push(SimulationEvent::BallCollisionArena { thrower_id, original_target_id: target_id, ball_id: *ball_id });
                    },
                    _ => ()
                }
            },
            (GameObjectType::Plate(plate_id), GameObjectType::Combatant(combatant_id)) | (GameObjectType::Combatant(combatant_id), GameObjectType::Plate(plate_id)) => {
                if evt.started() {
                    new_simulation_events.push(SimulationEvent::CombatantOnPlate { combatant_id: *combatant_id, plate_id: *plate_id })
                }

                if evt.stopped() {
                    new_simulation_events.push(SimulationEvent::CombatantOffPlate { combatant_id: *combatant_id, plate_id: *plate_id })
                }
            },
            (GameObjectType::Plate(_), _) | (_, GameObjectType::Plate(_)) => continue,
            (GameObjectType::BallSpawn, _) | (_, GameObjectType::BallSpawn) => continue,
            (GameObjectType::Barrier, _) | (_, GameObjectType::Barrier) => continue,
            (GameObjectType::Ball(_), GameObjectType::Ball(_)) => continue,
            (GameObjectType::Combatant(_), GameObjectType::Combatant(_)) => continue,
            (GameObjectType::Ball(ball_id), GameObjectType::Combatant(combatant_id)) | (GameObjectType::Combatant(combatant_id), GameObjectType::Ball(ball_id)) => {
                // We don't care if the ball and combatant have stopped colliding
                if evt.stopped() {
                    continue;
                }

                let ball_obj = balls.get(ball_id).expect("Received invalid ball ID");

                match ball_obj.state {
                    BallState::ThrownAtTarget { direction: _, thrower_id, target_id: _ } => {
                        // ZJ-TODO: check team of hit combatant, and only explode if enemy
                        new_simulation_events.push(SimulationEvent::BallCollisionEnemy { thrower_id, enemy_id: *combatant_id, ball_id: *ball_id });
                    },
                    _ => ()
                }
            }
        }
    }

    SimulationStage {
        execution_duration: start_time.elapsed(),
        pending_events: new_simulation_events
    }
}