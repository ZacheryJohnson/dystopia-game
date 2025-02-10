use std::sync::{Arc, Mutex};
use std::time::Instant;
use rapier3d::{na::vector, prelude::*};

use crate::{game_objects::{ball::{BallObject, BallState}, game_object::GameObject, game_object_type::GameObjectType}, game_state::GameState, game_tick::GameTickNumber};
use crate::simulation::simulation_stage::SimulationStage;
use super::{config::SimulationConfig, simulation_event::SimulationEvent};

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
        }

        if ball_object.is_dirty() {
            let mut game_state = game_state.lock().unwrap();
            let (rigid_body_set, _, _) = game_state.physics_sim.sets_mut();
            let ball_rb = rigid_body_set.get_mut(ball_object.rigid_body_handle().unwrap()).unwrap();

            events.push(SimulationEvent::BallPositionUpdate { ball_id, position: *ball_rb.translation() });
        }
    }

    {
        let mut game_state = game_state.lock().unwrap();
        let simulation_config = game_state.simulation_config.clone();

        for (_, ball_object) in &mut game_state.balls {
            decrease_charge(ball_object, &simulation_config);
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
) -> Option<SimulationEvent> {
    let BallState::Held { holder_id } = ball.state else {
        return None;
    };

    // ZJ-TODO
    // Don't love setting the ball's position to exactly the combatant's position but it works for now
    // Would love to actually figure out parenting one rigid body to another (joints?)

    let combatant_pos = {
        let game_state = game_state.lock().unwrap();
        let (rigid_body_set, _, _) = game_state.physics_sim.sets();

        let holding_combatant_object = game_state
            .combatants
            .get(&holder_id)
            .expect("failed to finding holder combatant object");

        rigid_body_set
            .get(holding_combatant_object.rigid_body_handle().unwrap())
            .unwrap()
            .translation()
            .to_owned()
    };

    Some(SimulationEvent::BallPositionUpdate { ball_id: ball.id, position: combatant_pos })
}

fn explode(
    ball: &BallObject,
    game_state: Arc<Mutex<GameState>>,
) -> Vec<SimulationEvent> {
    // Only handle balls in the Explode state
    let BallState::Explode = ball.state else {
        return vec![];
    };

    let ball_pos = {
        let game_state = game_state.lock().unwrap();
        let (rigid_body_set, _, _) = game_state.physics_sim.sets();

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
        SimulationEvent::BallExplosion { ball_id: ball.id, charge: ball.charge }
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
) -> Vec<SimulationEvent> {
    let BallState::Explode = ball_object.state else {
        return vec![];
    };

    let mut events = vec![];
    let game_state = game_state.lock().unwrap();
    let (rigid_body_set, _, _) = game_state.physics_sim.sets();
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

    events.push(SimulationEvent::BallExplosionForceApplied {
        ball_id: ball_object.id,
        combatant_id: *combatant_id,
        force_magnitude,
        force_direction
    });

    events
}

fn decrease_charge(ball: &mut BallObject, simulation_config: &SimulationConfig) {
    ball.charge = (ball.charge - simulation_config.ball_charge_decay_per_tick()).clamp(0.0, simulation_config.ball_charge_maximum());
}

fn try_freeze_slow_moving_ball(current_tick: GameTickNumber, ball_object: &mut BallObject, ball_rb: &mut RigidBody) {
    match ball_object.state {
        BallState::Explode | BallState::Held { .. } => return,
        _ => (),
    };

    const KINETIC_ENERGY_THRESHOLD: f32 = 3.0;
    if ball_rb.kinetic_energy() >= KINETIC_ENERGY_THRESHOLD {
        return;
    }

    // Ball is too slow - set it's velocity to zero to prevent further physics sim work
    ball_rb.set_linvel(vector![0.0, 0.0, 0.0], true);
    ball_object.change_state(current_tick, BallState::Idle);
    ball_object.is_dirty = false; // ZJ-TODO: handle this case in change_state; need to derive PartialEq
}