use std::collections::HashMap;

use dys_world::combatant::combatant::CombatantId;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rapier3d::prelude::*;

use crate::{game::Game, game_objects::{ball::{BallId, BallObject}, combatant::CombatantObject, game_object_type::GameObjectType}, game_tick::{GameTick, GameTickNumber}, physics_sim::PhysicsSim, simulation::simulate_tick};

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
        let mut physics_sim = PhysicsSim::new();
        let (rigid_body_set, collider_set) = physics_sim.sets();
        game.schedule_game.arena.lock().unwrap().register_features_physics(rigid_body_set, collider_set);

        let mut active_colliders = HashMap::new();

        // ZJ-TODO: move the following to arena init
        let ball_id = 1;
        let ball_object = BallObject::new(ball_id, 1, vector![30.0, 1.0, 30.0], rigid_body_set, collider_set);

        let ball_object_rb = rigid_body_set.get_mut(ball_object.rigid_body_handle).unwrap();
        ball_object_rb.apply_impulse(vector![75.0, 0.0, 55.0], true);

        active_colliders.insert(ball_object.collider_handle, GameObjectType::Ball(ball_id));

        let mut balls = HashMap::new();
        balls.insert(ball_id, ball_object);

        // ZJ-TODO: combatant init

        GameState {
            game,
            seed: seed.to_owned(),
            rng: Pcg64::from_seed(*seed),
            physics_sim,
            combatants: HashMap::new(),
            active_colliders,
            balls,
            home_points: 0,
            away_points: 0,
            current_tick: 0,
        }
    }

    pub fn tick(&mut self) -> GameTick {
        simulate_tick(self)
    }
}