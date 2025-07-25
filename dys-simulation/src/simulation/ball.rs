use std::sync::{Arc, Mutex};
use std::time::Instant;
use rapier3d::{na::vector, prelude::*};
use rapier3d::na::Vector3;
use crate::{game_objects::{ball::{BallObject, BallState}, game_object::GameObject, game_object_type::GameObjectType}, game_state::GameState};
use crate::simulation::simulation_event::PendingSimulationEvent;
use crate::simulation::simulation_stage::SimulationStage;
use super::{config::SimulationConfig, simulation_event::SimulationEvent};

/// Charge is an arbitrary metric to determine the "strength" of an explosion.
/// Charge itself is not a measure of Newtons applied to the combatant, but rather a multiplier.
/// The force modifier is a coefficient such that a ball with charge 1.0 will result in a
/// 100kg combatant being accelerated 5 units/second^2 (eg 500 Newtons).
const CHARGE_FORCE_MODIFIER: f32 = 500.0;

pub(crate) fn simulate_balls(game_state: Arc<Mutex<GameState>>) -> SimulationStage {
    let start_time = Instant::now();
    let mut events = vec![];

    let balls = {
        let game_state = game_state.lock().unwrap();
        game_state.balls.clone()
    };

    for (ball_id, ball_object) in balls {
        let explosion_simulation_events = explode(&ball_object, game_state.clone());
        events.extend(explosion_simulation_events);

        if let Some(event) = try_move_if_held(&ball_object, game_state.clone()) {
            events.push(event);
        } else {
            let mut game_state = game_state.lock().unwrap();
            let (rigid_body_set, _) = game_state.physics_sim.sets_mut();
            let ball_rb = rigid_body_set.get_mut(ball_object.rigid_body_handle().unwrap()).unwrap();

            events.push(PendingSimulationEvent(
                SimulationEvent::BallPositionUpdate {
                    ball_id,
                    position: *ball_rb.translation(),
                    charge: ball_object.charge,
                }
            ));
        }
    }

    {
        let mut game_state = game_state.lock().unwrap();
        let simulation_config = game_state.simulation_config.clone();

        for (_, ball_object) in &mut game_state.balls {
            increase_charge(ball_object, &simulation_config);
        }
    }

    SimulationStage {
        pending_events: events,
        execution_duration: start_time.elapsed(),
    }
}

fn try_move_if_held(
    ball: &BallObject,
    game_state: Arc<Mutex<GameState>>,
) -> Option<PendingSimulationEvent> {
    let BallState::Held { holder_id } = ball.state else {
        return None;
    };

    // ZJ-TODO
    // Don't love setting the ball's position to exactly the combatant's position but it works for now
    // Would love to actually figure out parenting one rigid body to another (joints?)

    let held_by_combatant_pos = {
        let game_state = game_state.lock().unwrap();
        let (rigid_body_set, _) = game_state.physics_sim.sets();

        let holding_combatant_object = game_state
            .combatants
            .get(&holder_id)
            .expect("failed to finding holder combatant object");

        let forward_isometry = holding_combatant_object.forward_isometry(rigid_body_set);
        let outside_of_geometry_dist = holding_combatant_object.radius() + ball.radius();
        let new_ball_position_offset = Vector3::z() * outside_of_geometry_dist;
        forward_isometry.translation.vector + forward_isometry.transform_vector(&new_ball_position_offset)
    };

    Some(PendingSimulationEvent(
        SimulationEvent::BallPositionUpdate {
            ball_id: ball.id,
            position: held_by_combatant_pos,
            charge: ball.charge,
        }
    ))
}

fn explode(
    ball: &BallObject,
    game_state: Arc<Mutex<GameState>>,
) -> Vec<PendingSimulationEvent> {
    // Only handle balls in the Explode state
    let BallState::Explode = ball.state else {
        return vec![];
    };

    let ball_pos = {
        let game_state = game_state.lock().unwrap();
        let (rigid_body_set, _) = game_state.physics_sim.sets();

        rigid_body_set
            .get(ball.rigid_body_handle().unwrap())
            .unwrap()
            .translation()
            .to_owned()
    };
    
    const EXPLOSION_CYLINDER_HEIGHT: f32 = 30.0;
    let explosion_radius = ball.charge * 0.3; // ZJ-TODO: figure out explosion radius as compared to charge
    let explosion_shape = Cylinder::new(EXPLOSION_CYLINDER_HEIGHT, explosion_radius);
    let explosion_pos = Isometry::new(ball_pos, vector![0.0, 0.0, 0.0]);
    let query_filter = QueryFilter::only_dynamic()
        .exclude_sensors();

    // ZJ-TODO: use InteractionGroups to get only combatants and ignore everything else
    let mut affected_colliders = vec![];

    {
        let mut game_state = game_state.lock().unwrap();
        let (query_pipeline, rigid_body_set, collider_set) = game_state.physics_sim.query_pipeline_and_sets();

        query_pipeline.intersections_with_shape(rigid_body_set, collider_set, &explosion_pos, &explosion_shape, query_filter, |handle| {
            affected_colliders.push(handle);
            true // return true to continue iterating over collisions
        });
    }

    let mut events = vec![
        PendingSimulationEvent(SimulationEvent::BallExplosion { ball_id: ball.id, charge: ball.charge })
    ];

    for collider_handle in affected_colliders {
        let new_events = apply_explosion_forces(
            game_state.clone(),
            collider_handle,
            ball,
        );

        events.extend(new_events);
    }

    events
}

fn apply_explosion_forces(
    game_state: Arc<Mutex<GameState>>,
    collider_handle: ColliderHandle,
    ball_object: &BallObject,
) -> Vec<PendingSimulationEvent> {
    let BallState::Explode = ball_object.state else {
        return vec![];
    };

    let mut events = vec![];
    let game_state = game_state.lock().unwrap();
    let (rigid_body_set, _) = game_state.physics_sim.sets();
    let GameObjectType::Combatant(combatant_id) = game_state.active_colliders.get(&collider_handle).unwrap() else {
        return vec![];
    };

    let combatant_rigid_body_handle = game_state.combatants.get(combatant_id).unwrap().rigid_body_handle;
    let combatant_pos = rigid_body_set.get(combatant_rigid_body_handle).unwrap().translation();
    let ball_pos = rigid_body_set.get(ball_object.rigid_body_handle().unwrap()).unwrap().translation();

    // Magnitude of the explosion force is the charge of the ball divided by the square distance
    // This means direct impacts will apply a LOT of force, while nearby combatants will take exponentially less per unit away
    let position_difference = combatant_pos - ball_pos;
    let force_direction = vector![position_difference.x, 0.0, position_difference.z].normalize();
    let force_magnitude = ball_object.charge * CHARGE_FORCE_MODIFIER / position_difference.magnitude().powi(2);

    events.push(PendingSimulationEvent(
        SimulationEvent::BallExplosionForceApplied {
            ball_id: ball_object.id,
            combatant_id: *combatant_id,
            force_magnitude,
            force_direction
        }
    ));

    // ZJ-TODO: this should be handled elsewhere
    events.push(PendingSimulationEvent(
        SimulationEvent::CombatantStunned {
            combatant_id: *combatant_id,
            start: true
        }
    ));

    events
}

fn increase_charge(ball: &mut BallObject, simulation_config: &SimulationConfig) {
    if matches!(ball.state, BallState::ThrownAtTarget { .. }) {
        // Only increase the charge of balls flying in the air
        ball.charge = (ball.charge + simulation_config.ball_charge_increase_per_tick())
            .clamp(0.0, simulation_config.ball_charge_maximum());
    }
}
