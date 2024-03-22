use std::collections::HashMap;

use dys_world::combatant::combatant::CombatantId;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use crate::{game::Game, game_objects::{ball::{BallId, BallObject}, combatant::CombatantObject}, game_tick::{GameTick, GameTickNumber}, simulation::simulate_tick};

pub type SeedT = [u8; 32];

pub struct Vector3(f32, f32, f32);

pub struct GameState {
    pub game: Game,
    pub seed: SeedT,
    pub rng: Pcg64,
    pub combatants: HashMap<CombatantId, CombatantObject>,
    pub balls: HashMap<BallId, BallObject>,
    pub home_points: u16,
    pub away_points: u16,
    pub current_tick: GameTickNumber,
}

impl GameState {
    pub fn from_game(game: Game) -> GameState {
        let seed = rand::thread_rng().gen::<SeedT>();
        GameState {
            game,
            seed,
            rng: Pcg64::from_seed(seed),
            combatants: HashMap::new(),
            balls: HashMap::new(),
            home_points: 0,
            away_points: 0,
            current_tick: 0,
        }
    }

    pub fn from_game_seeded(game: Game, seed: &SeedT) -> GameState {
        GameState {
            game,
            seed: seed.to_owned(),
            rng: Pcg64::from_seed(*seed),
            combatants: HashMap::new(),
            balls: HashMap::new(),
            home_points: 0,
            away_points: 0,
            current_tick: 0,
        }
    }

    pub fn tick(&mut self) -> GameTick {
        simulate_tick(self)
    }
}