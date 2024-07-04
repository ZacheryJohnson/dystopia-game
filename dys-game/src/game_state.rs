use std::collections::HashMap;

use dys_world::{arena::{ball_spawn::ArenaBallSpawn, barrier::ArenaBarrier, combatant_start::ArenaCombatantStart, feature::ArenaFeature, navmesh::{ArenaNavmesh, ArenaNavmeshConfig}, plate::{ArenaPlate, PlateId}}, combatant::combatant::CombatantId};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rapier3d::prelude::*;

use crate::{game::Game, game_objects::{ball::{BallId, BallObject, BallState}, combatant::{CombatantObject, TeamAlignment}, game_object::GameObject, game_object_type::GameObjectType, plate::PlateObject}, game_tick::{GameTick, GameTickNumber}, physics_sim::PhysicsSim, simulation::{config::SimulationConfig, simulate_tick}, targeting::get_throw_vector_towards_target};

pub type SeedT = [u8; 32];

pub struct GameState {
    pub game: Game,
    pub seed: SeedT,
    pub rng: Pcg64,
    pub physics_sim: PhysicsSim,
    pub combatants: HashMap<CombatantId, CombatantObject>,
    pub balls: HashMap<BallId, BallObject>,
    pub plates: HashMap<PlateId, PlateObject>,
    pub active_colliders: HashMap<ColliderHandle, GameObjectType>,
    pub home_points: u16,
    pub away_points: u16,
    pub current_tick: GameTickNumber,
    pub simulation_config: SimulationConfig,
    pub arena_navmesh: ArenaNavmesh,
}

fn get_game_object_type_from_feature(feature: &Box<dyn ArenaFeature>) -> GameObjectType {
    if feature.as_any().downcast_ref::<ArenaBarrier>().is_some() {
        return GameObjectType::Barrier;
    }

    if feature.as_any().downcast_ref::<ArenaBallSpawn>().is_some() {
        return GameObjectType::BallSpawn;
    }

    if let Some(plate) = feature.as_any().downcast_ref::<ArenaPlate>() {
        return GameObjectType::Plate(plate.id);
    }

    panic!("unknown game object type for feature");
}

impl GameState {
    pub fn from_game(game: Game) -> GameState {
        let seed = rand::thread_rng().gen::<SeedT>();
        GameState::from_game_seeded(game, &seed)
    }

    pub fn from_game_seeded(game: Game, seed: &SeedT) -> GameState {
        let current_tick = 0;

        let simulation_config = SimulationConfig::default();
        let mut physics_sim = PhysicsSim::new(simulation_config.ticks_per_second());
        let gravity_y = physics_sim.gravity_y();
        let (rigid_body_set, collider_set) = physics_sim.sets_mut();

        let mut active_colliders = HashMap::new();
        let mut balls = HashMap::new();
        let mut combatants = HashMap::new();
        let mut plates = HashMap::new();

        {
            let arena = game.schedule_game.arena.lock().unwrap();
            for feature in arena.all_features() {
                if let Some(rigid_body) = feature.build_rigid_body() {
                    let rigid_body_handle = rigid_body_set.insert(rigid_body);
                    if let Some(collider) = feature.build_collider() {
                        let collider_handle = collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);
                        let game_object_type = get_game_object_type_from_feature(feature);
                        match game_object_type {
                            GameObjectType::Plate(plate_id) => plates.insert(plate_id, PlateObject::new(collider_handle)),
                            _ => None,
                        };

                        active_colliders.insert(collider_handle, game_object_type);
                    }
                } else if let Some(collider) = feature.build_collider() {
                    let collider_handle = collider_set.insert(collider);
                    let game_object_type = get_game_object_type_from_feature(feature);
                    match game_object_type {
                        GameObjectType::Plate(plate_id) => plates.insert(plate_id, PlateObject::new(collider_handle)),
                        _ => None,
                    };

                    active_colliders.insert(collider_handle, game_object_type);
                }
            }

            let mut ball_id = 0;

            for ball_spawn in arena.features::<ArenaBallSpawn>() {
                ball_id += 1;
                let ball_object = BallObject::new(ball_id, current_tick, *ball_spawn.origin() + vector![0.0, 1.0, 0.0], rigid_body_set, collider_set);

                active_colliders.insert(ball_object.collider_handle().expect("ball game objects must have collider handles"), GameObjectType::Ball(ball_id));

                balls.insert(ball_id, ball_object);
            }
        }

        {
            let mut home_combatants = { game.schedule_game.home_team.lock().unwrap().combatants.clone() };
            let mut away_combatants = { game.schedule_game.away_team.lock().unwrap().combatants.clone() };

            let arena = game.schedule_game.arena.lock().unwrap();
            let combatant_starts = arena.features::<ArenaCombatantStart>();

            let mut combatant_id = 0;
            for player_start in combatant_starts {
                combatant_id += 1;
                let position = player_start.origin() + vector![0.0, 2.5, 0.0];

                let team_combatants = if player_start.is_home_team { &mut home_combatants } else { &mut away_combatants };
                let Some(combatant) = team_combatants.pop() else {
                    // This may not be an error case if we allow more starts than combatants
                    tracing::info!("failed to pop combatant for empty player start");
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
            let (_, ball_obj) = balls.iter_mut().next().unwrap();
            let ball_rb = rigid_body_set.get(ball_obj.rigid_body_handle().expect("balls should have rigid bodies")).expect("failed to find ball rigid body");
            let ball_pos = ball_rb.translation();

            let (thrower_id, _) = combatants
                .iter().find(|(_, combatant_obj)| combatant_obj.team == TeamAlignment::Home)
                .expect("failed to find home team combatant");
            
            let (target_id, target_obj) = combatants
                .iter().find(|(_, combatant_obj)| combatant_obj.team == TeamAlignment::Away)
                .expect("failed to find away team combatant");
            let target_pos = rigid_body_set
                .get(target_obj.rigid_body_handle().expect("combatants should have rigid bodies"))
                .expect("failed to get target rigid body")
                .translation();

            let throw_speed_units_per_sec = 25.0; // ZJ-TODO: read from combatant stats
            let accuracy = 1.0_f32.clamp(0.0, 1.0); // ZJ-TODO: read from combatant stats
            let y_axis_gravity = gravity_y;
            let impulse = get_throw_vector_towards_target(target_pos, ball_pos, throw_speed_units_per_sec, accuracy, y_axis_gravity);

            let new_state = BallState::ThrownAtTarget { 
                direction: impulse, 
                thrower_id: *thrower_id,
                target_id: *target_id,
            };

            ball_obj.charge = 80.0; // ZJ-TODO: don't add arbitrary charge
            ball_obj.change_state(current_tick, new_state);

            let mut_ball_rb = rigid_body_set.get_mut(ball_obj.rigid_body_handle().expect("balls should have rigid bodies")).expect("failed to find ball rigid body");
            mut_ball_rb.apply_impulse(impulse, true);
        }

        let arena_navmesh = ArenaNavmesh::new_from(game.schedule_game.arena.clone(), ArenaNavmeshConfig::default());

        GameState {
            game,
            seed: seed.to_owned(),
            rng: Pcg64::from_seed(*seed),
            physics_sim,
            combatants,
            active_colliders,
            balls,
            plates,
            home_points: 0,
            away_points: 0,
            current_tick,
            simulation_config,
            arena_navmesh
        }
    }

    pub fn tick(&mut self) -> GameTick {
        simulate_tick(self)
    }
}