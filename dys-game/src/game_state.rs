use std::collections::HashMap;

use dys_world::combatant::combatant::CombatantId;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rapier3d::prelude::*;

use crate::{game::Game, game_objects::{ball::{BallId, BallObject, BallState}, combatant::{CombatantObject, TeamAlignment}, game_object::GameObject, game_object_type::GameObjectType}, game_tick::{GameTick, GameTickNumber}, physics_sim::PhysicsSim, simulation::{simulate_tick, TICKS_PER_SECOND}};
use dys_world::arena::feature::ArenaFeature;

pub type SeedT = [u8; 32];

pub struct GameState {
    pub game: Game,
    pub seed: SeedT,
    pub rng: Pcg64,
    pub physics_sim: PhysicsSim,
    pub combatants: HashMap<CombatantId, CombatantObject>,
    pub balls: HashMap<BallId, BallObject>,
    pub active_colliders: HashMap<ColliderHandle, GameObjectType>,
    pub home_points: u16,
    pub away_points: u16,
    pub current_tick: GameTickNumber,
}

impl GameState {
    pub fn from_game(game: Game) -> GameState {
        let seed = rand::thread_rng().gen::<SeedT>();
        GameState::from_game_seeded(game, &seed)
    }

    pub fn from_game_seeded(game: Game, seed: &SeedT) -> GameState {
        let current_tick = 0;

        let mut physics_sim = PhysicsSim::new(TICKS_PER_SECOND as f32);
        let (rigid_body_set, collider_set) = physics_sim.sets();

        let mut active_colliders = HashMap::new();
        let mut balls = HashMap::new();
        let mut combatants = HashMap::new();

        {
            let arena = game.schedule_game.arena.lock().unwrap();
            for feature in &arena.features {
                if let Some(rigid_body) = feature.build_rigid_body() {
                    let rigid_body_handle = rigid_body_set.insert(rigid_body);
                    if let Some(collider) = feature.build_collider() {
                        let collider_handle = collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);
                        active_colliders.insert(collider_handle, GameObjectType::Wall); // ZJ-TODO: don't hardcode wall - this could be a plate
                    }
                } else {
                    if let Some(collider) = feature.build_collider() {
                        let collider_handle = collider_set.insert(collider);
                        active_colliders.insert(collider_handle, GameObjectType::Wall); // ZJ-TODO: don't hardcode wall - this could be a plate
                    }
                }
            }

            // ZJ-TODO: move the following to arena init
            let mut ball_id = 0;

            for ball_spawn in arena.ball_spawns() {
                ball_id += 1;
                let ball_object = BallObject::new(ball_id, current_tick, *ball_spawn.origin() + vector![0.0, 1.0, 0.0], rigid_body_set, collider_set);

                active_colliders.insert(ball_object.collider_handle().expect("ball game objects must have collider handles"), GameObjectType::Ball(ball_id));

                balls.insert(ball_id, ball_object);
            }
        }

        // ZJ-TODO: combatant init
        {
            let mut home_combatants = { game.schedule_game.home_team.lock().unwrap().combatants.clone() };
            let mut away_combatants = { game.schedule_game.away_team.lock().unwrap().combatants.clone() };

            let arena = game.schedule_game.arena.lock().unwrap();
            let combatant_starts = arena.combatant_starts();

            let mut combatant_id = 0;
            for player_start in combatant_starts {
                combatant_id += 1;
                let position = player_start.origin() + vector![0.0, 2.5, 0.0];

                let team_combatants = if player_start.is_home_team { &mut home_combatants } else { &mut away_combatants };
                let Some(combatant) = team_combatants.pop() else {
                    // This may not be an error case if we allow more starts than combatants
                    println!("failed to pop combatant for empty player start");
                    continue;
                };

                let team_alignment = if player_start.is_home_team { TeamAlignment::Home } else { TeamAlignment::Away };

                let combatant_object = CombatantObject::new(combatant_id, combatant, position, team_alignment, rigid_body_set, collider_set);
                active_colliders.insert(combatant_object.collider_handle().expect("combatant game objects must have collider handles"), GameObjectType::Combatant(combatant_id));
                combatants.insert(combatant_id, combatant_object);
            }
        }

        // ZJ-TODO: delete this block. Testing ball collisions
        {
            let (ball_id, ball_obj) = balls.iter_mut().next().unwrap();
            let ball_rb = rigid_body_set.get(ball_obj.rigid_body_handle().expect("balls should have rigid bodies")).expect("failed to find ball rigid body");
            let ball_pos = ball_rb.translation();

            let (thrower_id, thrower_obj) = combatants
                .iter()
                .filter(|(_, combatant_obj)| combatant_obj.team == TeamAlignment::Home)
                .next()
                .expect("failed to find home team combatant");
            
            let (target_id, target_obj) = combatants
                .iter()
                .filter(|(_, combatant_obj)| combatant_obj.team == TeamAlignment::Away)
                .next()
                .expect("failed to find away team combatant");
            let target_pos = rigid_body_set
                .get(target_obj.rigid_body_handle().expect("combatants should have rigid bodies"))
                .expect("failed to get target rigid body")
                .translation();

            // ZJ-TODO: extract the below into a generic "throw_ball" function. There's a lot of reusable physics math in here

            // We're telekinetically throwing the ball in this case - the ball would otherwise be parented to the thrower.
            let difference_vector = target_pos - ball_pos;
            let difference_distance = difference_vector.magnitude();
            let throw_speed_units_per_sec = 25.0; // This particular throw will go 25 units per second; should be read from the thrower's stats
            let total_travel_time_sec = difference_distance / throw_speed_units_per_sec;
            let gravity_adjustment_magnitude = (4.905 * (total_travel_time_sec.powi(2)) + difference_vector.y) / total_travel_time_sec;
            let impulse_direction = vector![difference_vector.x, 0.0, difference_vector.z].normalize();

            let impulse = (impulse_direction * throw_speed_units_per_sec) + vector![0.0, gravity_adjustment_magnitude, 0.0]; 

            let new_state = BallState::ThrownAtTarget { 
                direction: impulse.clone(), 
                thrower_id: *thrower_id,
                target_id: *target_id,
            };

            ball_obj.change_state(current_tick, new_state);

            let mut_ball_rb = rigid_body_set.get_mut(ball_obj.rigid_body_handle().expect("balls should have rigid bodies")).expect("failed to find ball rigid body");
            mut_ball_rb.apply_impulse(impulse, true);
        }

        GameState {
            game,
            seed: seed.to_owned(),
            rng: Pcg64::from_seed(*seed),
            physics_sim,
            combatants,
            active_colliders,
            balls,
            home_points: 0,
            away_points: 0,
            current_tick,
        }
    }

    pub fn tick(&mut self) -> GameTick {
        simulate_tick(self)
    }
}