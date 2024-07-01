use std::collections::HashMap;

use rapier3d::{na::vector, prelude::*};

use crate::{game_objects::{ball::{BallObject, BallState}, combatant::CombatantObject, game_object::GameObject, game_object_type::GameObjectType}, game_state::GameState, game_tick::GameTickNumber};

use super::{config::SimulationConfig, simulation_event::SimulationEvent};

const CHARGE_FORCE_MODIFIER: f32 = 2.0;

pub(crate) fn simulate_balls(game_state: &mut GameState) -> Vec<SimulationEvent> {
    let mut events = vec![];

    let (query_pipeline, rigid_body_set, collider_set) = game_state.physics_sim.query_pipeline_and_sets();
    for (ball_id, ball_object) in &mut game_state.balls {
        { 
            let (explosion_simulation_events, affected_colliders) = explode(ball_object, query_pipeline, rigid_body_set, collider_set);
            events.extend(explosion_simulation_events);

            let explosion_force_simulation_events = apply_explosion_forces(game_state.current_tick, affected_colliders, ball_object, &game_state.active_colliders, &mut game_state.combatants, rigid_body_set);
            events.extend(explosion_force_simulation_events);
            decrease_charge(ball_object, &game_state.simulation_config);
        }
        {
            let ball_rb_handle = ball_object.rigid_body_handle().expect("ball should have a valid rigidbody handle");
            let ball_rb: &mut RigidBody = rigid_body_set.get_mut(ball_rb_handle).expect("ball rigid bodies should be registered with main set");
    
            try_freeze_slow_moving_ball(game_state.current_tick, ball_object, ball_rb);
    
            if ball_object.is_dirty() {
                events.push(SimulationEvent::BallPositionUpdate { ball_id: *ball_id, position: *ball_rb.translation() });
            }
        }
    }

    events
}

fn explode(
    ball: &mut BallObject,
    query_pipeline: &QueryPipeline,
    rigid_body_set: &RigidBodySet,
    collider_set: &ColliderSet,
) -> (Vec<SimulationEvent>, Vec<ColliderHandle>) {
    // Only handle balls in the Explode state
    let BallState::Explode = ball.state else {
        return (vec![], vec![]);
    };

    let ball_pos = rigid_body_set.get(ball.rigid_body_handle().unwrap()).unwrap().translation();

    const EXPLOSION_CYLINDER_HEIGHT: f32 = 30.0;
    let explosion_radius = ball.charge * 1.0; // ZJ-TODO: figure out explosion radius as compared to charge
    let explosion_shape = Cylinder::new(EXPLOSION_CYLINDER_HEIGHT, explosion_radius);
    let explosion_pos = Isometry::new(ball_pos.to_owned(), vector![0.0, 0.0, 0.0]);
    let query_filter = QueryFilter::only_dynamic()
        .exclude_sensors();
    // ZJ-TODO: use InteractionGroups to get only combatants and ignore everything else

    let mut affected_colliders = vec![];
    query_pipeline.intersections_with_shape(rigid_body_set, collider_set, &explosion_pos, &explosion_shape, query_filter, |handle| {
        affected_colliders.push(handle);
        true // return true to continue iterating over collisions
    });

    (vec![SimulationEvent::BallExplosion { ball_id: ball.id, charge: ball.charge }], affected_colliders)
}

fn apply_explosion_forces(
    current_tick: GameTickNumber,
    affected_colliders: Vec<ColliderHandle>,
    ball_object: &mut BallObject,
    active_colliders: &HashMap<ColliderHandle, GameObjectType>,
    combatants: &mut HashMap<u64, CombatantObject>,
    rigid_body_set: &mut RigidBodySet,
) -> Vec<SimulationEvent> {
    let BallState::Explode = ball_object.state else {
        return vec![];
    };

    let mut events = vec![];
    for collider in affected_colliders {
        let GameObjectType::Combatant(combatant_id) = active_colliders.get(&collider).unwrap() else {
            continue;
        };

        let combatant = combatants.get_mut(combatant_id).unwrap();
        let combatant_pos = rigid_body_set.get(combatant.rigid_body_handle().unwrap()).unwrap().translation();
        let ball_pos = rigid_body_set.get(ball_object.rigid_body_handle().unwrap()).unwrap().translation();

        // Magnitude of the explosion force is the charge of the ball divided by the square distance
        // This means direct impacts will apply a LOT of force, while nearby combatants will take exponentially less per unit away
        let position_difference = ball_pos - combatant_pos;
        let force_direction = vector![position_difference.x, 0.0, position_difference.z];
        let force_magnitude = ball_object.charge * CHARGE_FORCE_MODIFIER / (force_direction.magnitude() + 1.0);

        combatant.apply_explosion_force(current_tick, force_magnitude, force_direction.normalize(), rigid_body_set);
        events.push(SimulationEvent::BallExplosionForceApplied { ball_id: ball_object.id, combatant_id: *combatant_id, force_magnitude, force_direction });
    }

    // After exploding, reset charge, make ball idle
    // ZJ-TODO: delete ball, spawn new one, etc
    ball_object.charge = 0.0;
    ball_object.change_state(current_tick, BallState::Idle);

    events
}

fn decrease_charge(ball: &mut BallObject, simulation_config: &SimulationConfig) {
    ball.charge = (ball.charge - simulation_config.ball_charge_decay_per_tick()).clamp(0.0, simulation_config.ball_charge_maximum());
}

fn try_freeze_slow_moving_ball(current_tick: GameTickNumber, ball_object: &mut BallObject, ball_rb: &mut RigidBody) {
    match ball_object.state {
        BallState::Explode => return,
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