use std::f32::consts::FRAC_PI_2;
use std::sync::{Arc, Mutex};
use indexmap::IndexMap;
use dys_world::{arena::{ball_spawn::ArenaBallSpawn, barrier::ArenaBarrier, combatant_start::ArenaCombatantStart, feature::ArenaFeature, navmesh::{ArenaNavmesh, ArenaNavmeshConfig}, plate::{ArenaPlate, PlateId}}};
use rand::{random, SeedableRng};
use rand_pcg::Pcg64;
use rapier3d::prelude::*;
use dys_world::arena::Arena;
use dys_world::combatant::instance::CombatantInstanceId;
use crate::{game::Game, game_objects::{ball::{BallId, BallObject}, combatant::{CombatantObject, TeamAlignment}, game_object::GameObject, game_object_type::GameObjectType, plate::PlateObject}, game_tick::GameTickNumber, physics_sim::PhysicsSim, simulation::config::SimulationConfig};
use crate::game_objects::combatant::CombatantId;

pub type SeedT = [u8; 32];

// IndexMap is a crate that preserves order of insertions.
// This is *critical* for ensuring that simulations are deterministic and repeatable.

pub type CombatantsMapT = IndexMap<CombatantId, CombatantObject>;
pub type BallsMapT = IndexMap<BallId, BallObject>;
pub type PlatesMapT = IndexMap<PlateId, PlateObject>;
pub type CollidersMapT = IndexMap<ColliderHandle, GameObjectType>;

pub struct GameState {
    pub game: Game,
    pub seed: SeedT,
    pub rng: Pcg64,
    pub physics_sim: PhysicsSim,
    pub combatants: CombatantsMapT,
    pub combatant_id_to_instance_id: IndexMap<CombatantId, CombatantInstanceId>,
    pub balls: BallsMapT,
    pub plates: PlatesMapT,
    pub active_colliders: CollidersMapT,
    pub home_points: u16,
    pub away_points: u16,
    pub current_tick: GameTickNumber,
    pub simulation_config: SimulationConfig,
    pub arena_navmesh: ArenaNavmesh,
}

fn get_game_object_type_from_feature(feature: &dyn ArenaFeature) -> GameObjectType {
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
        let seed = random::<SeedT>();
        GameState::from_game_seeded(game, &seed)
    }

    pub fn from_game_seeded(game: Game, seed: &SeedT) -> GameState {
        let current_tick = 0;

        let simulation_config = SimulationConfig::default();
        let mut physics_sim = PhysicsSim::new(simulation_config.ticks_per_second());
        let (rigid_body_set, collider_set, _) = physics_sim.sets_mut();

        let mut active_colliders = CollidersMapT::new();
        let mut balls = BallsMapT::new();
        let mut combatants = CombatantsMapT::new();
        let mut combatant_id_to_instance_id = IndexMap::new();
        let mut plates = PlatesMapT::new();

        {
            // let arena = game.match_instance.arena.lock().unwrap();
            let arena = Arena::new_with_testing_defaults();
            for feature in arena.all_features() {
                if let Some(rigid_body) = feature.build_rigid_body() {
                    let rigid_body_handle = rigid_body_set.insert(rigid_body);
                    if let Some(collider) = feature.build_collider() {
                        let collider_handle = collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);
                        let game_object_type = get_game_object_type_from_feature(feature);
                        if let GameObjectType::Plate(plate_id) = game_object_type {
                            plates.insert(plate_id, PlateObject::new(plate_id, collider_handle));
                        };

                        active_colliders.insert(collider_handle, game_object_type);
                    }
                } else if let Some(collider) = feature.build_collider() {
                    let collider_handle = collider_set.insert(collider);
                    let game_object_type = get_game_object_type_from_feature(feature);
                    match game_object_type {
                        GameObjectType::Plate(plate_id) => plates.insert(plate_id, PlateObject::new(plate_id, collider_handle)),
                        _ => None,
                    };

                    active_colliders.insert(collider_handle, game_object_type);
                }
            }

            let mut ball_id = 0;

            for ball_spawn in arena.features::<ArenaBallSpawn>() {
                ball_id += 1;
                let ball_object = BallObject::new(ball_id, current_tick, *ball_spawn.origin(), rigid_body_set, collider_set);

                active_colliders.insert(ball_object.collider_handle().expect("ball game objects must have collider handles"), GameObjectType::Ball(ball_id));

                balls.insert(ball_id, ball_object);
            }
        }

        {
            let mut home_combatants = { game.match_instance.home_team.lock().unwrap().combatants.clone() };
            let mut away_combatants = { game.match_instance.away_team.lock().unwrap().combatants.clone() };

            // let arena = game.match_instance.arena.lock().unwrap();
            let arena = Arena::new_with_testing_defaults();
            let combatant_starts = arena.features::<ArenaCombatantStart>();

            let mut combatant_id = 0;
            for player_start in combatant_starts {
                combatant_id += 1;

                let team_combatants = if player_start.is_home_team { &mut home_combatants } else { &mut away_combatants };
                let Some(combatant) = team_combatants.pop() else {
                    // This may not be an error case if we allow more starts than combatants
                    tracing::info!("failed to pop combatant for empty player start");
                    continue;
                };

                let team_alignment = if player_start.is_home_team { TeamAlignment::Home } else { TeamAlignment::Away };

                let combatant_object = CombatantObject::new(
                    combatant_id,
                    combatant,
                    *player_start.origin(),
                    AngVector::new(0.0, if team_alignment == TeamAlignment::Home { FRAC_PI_2 } else { -FRAC_PI_2 }, 0.0),
                    team_alignment,
                    rigid_body_set,
                    collider_set);
                active_colliders.insert(combatant_object.collider_handle().expect("combatant game objects must have collider handles"), GameObjectType::Combatant(combatant_id));
                combatant_id_to_instance_id.insert(combatant_id, combatant_object.combatant.lock().unwrap().id);
                combatants.insert(combatant_id, combatant_object);
            }
        }

        let arena_navmesh = ArenaNavmesh::new_from(
            Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
            // game.match_instance.arena.clone(),
            ArenaNavmeshConfig {
                unit_resolution: 1.0
            }
        );

        GameState {
            game,
            seed: seed.to_owned(),
            rng: Pcg64::from_seed(*seed),
            physics_sim,
            combatants,
            combatant_id_to_instance_id,
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

    pub fn home_and_away_combatants(&self) -> [Vec<CombatantObject>; 2] {
        let home_team = self.combatants
            .clone()
            .into_iter()
            .filter(|(_, combatant_object)| combatant_object.team == TeamAlignment::Home)
            .map(|(_, combatant_object)| combatant_object)
            .collect::<Vec<_>>();

        let away_team = self.combatants
            .clone()
            .into_iter()
            .filter(|(_, combatant_object)| combatant_object.team == TeamAlignment::Away)
            .map(|(_, combatant_object)| combatant_object)
            .collect::<Vec<_>>();

        [home_team, away_team]
    }

    pub fn team_combatants(&self, team: TeamAlignment) -> Vec<CombatantObject> {
        let [home_combatants, away_combatants] = self.home_and_away_combatants();
        match team {
            TeamAlignment::Home => home_combatants,
            TeamAlignment::Away => away_combatants,
        }
    }

    pub fn is_scoring_tick(&self) -> bool {
        self.current_tick % self.simulation_config.ticks_per_second() == 0
    }
}