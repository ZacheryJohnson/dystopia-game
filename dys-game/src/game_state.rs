use std::collections::HashMap;

use dys_world::combatant::combatant::CombatantId;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rapier3d::{na::Vector3, prelude::*};

use crate::{game::Game, game_objects::{ball::{BallId, BallObject}, combatant::CombatantObject}, game_tick::{GameTick, GameTickNumber}, simulation::simulate_tick};

pub type SeedT = [u8; 32];

pub struct GameState {
    pub game: Game,
    pub seed: SeedT,
    pub rng: Pcg64,
    pub physics_sim: PhysicsSim,
    pub combatants: HashMap<CombatantId, CombatantObject>,
    pub balls: HashMap<BallId, BallObject>,
    pub home_points: u16,
    pub away_points: u16,
    pub current_tick: GameTickNumber,
}

pub struct PhysicsSim {
    gravity: Vector3<f32>,
    integration_params: IntegrationParameters,
    pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    event_handler: (),
}

impl PhysicsSim {
    pub fn new() -> PhysicsSim {
        let mut integration_params = IntegrationParameters::default();
        integration_params.dt = 1.0 / 5.0; // ZJ-TODO: read from TICKS_PER_SECOND

        PhysicsSim {
            gravity: vector![0.0, -9.81, 0.0],
            integration_params,
            pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    pub fn sets(&mut self) -> (&mut RigidBodySet, &mut ColliderSet) {
        (&mut self.rigid_body_set, &mut self.collider_set)
    }

    pub fn tick(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        )
    }
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

        // ZJ-TODO: move the following to arena init
        let ball_object = BallObject::new(1, 1, vector![30.0, 1.0, 30.0], rigid_body_set, collider_set);

        let ball_object_rb = rigid_body_set.get_mut(ball_object.rigid_body_handle).unwrap();
        ball_object_rb.apply_impulse(vector![75.0, 0.0, 55.0], true);

        let mut balls = HashMap::new();
        balls.insert(1, ball_object);

        // ZJ-TODO: combatant init

        GameState {
            game,
            seed: seed.to_owned(),
            rng: Pcg64::from_seed(*seed),
            physics_sim,
            combatants: HashMap::new(),
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