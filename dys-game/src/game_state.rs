use std::collections::HashMap;

use dys_world::combatant::combatant::CombatantId;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rapier3d::prelude::*;

use crate::{game::Game, game_objects::{ball::{BallId, BallObject}, combatant::CombatantObject, game_object::GameObject, game_object_type::GameObjectType}, game_tick::{GameTick, GameTickNumber}, physics_sim::PhysicsSim, simulation::simulate_tick};
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

        let mut physics_sim = PhysicsSim::new();
        let (rigid_body_set, collider_set) = physics_sim.sets();

        let mut active_colliders = HashMap::new();
        let mut balls = HashMap::new();
        let mut combatants = HashMap::new();

        {
            let arena = game.schedule_game.arena.lock().unwrap();
            arena.register_features_physics(rigid_body_set, collider_set);

            // ZJ-TODO: move the following to arena init
            let mut ball_id = 0;

            for ball_spawn in arena.ball_spawns() {
                ball_id += 1;
                let ball_object = BallObject::new(ball_id, current_tick, *ball_spawn.origin(), rigid_body_set, collider_set);

                active_colliders.insert(ball_object.collider_handle, GameObjectType::Ball(ball_id));

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
                let position = player_start.origin().to_owned();

                let team_combatants = if player_start.is_home_team { &mut home_combatants } else { &mut away_combatants };
                let Some(combatant) = team_combatants.pop() else {
                    // This may not be an error case if we allow more starts than combatants
                    println!("failed to pop combatant for empty player start");
                    continue;
                };

                let combatant_object = CombatantObject::new(combatant_id, combatant, position, rigid_body_set, collider_set);
                active_colliders.insert(combatant_object.collider_handle().expect("combatant game objects must have collider handles"), GameObjectType::Combatant(combatant_id));
                combatants.insert(combatant_id, combatant_object);
            }
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