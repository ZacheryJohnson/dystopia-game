use std::fmt::Debug;
use dyn_clone::DynClone;
use rapier3d::na::Isometry3;
use rapier3d::prelude::*;
use crate::ai::belief::ExpiringBelief;
use crate::game_state::{BallsMapT, CollidersMapT, CombatantsMapT};
use crate::game_tick::GameTickNumber;

pub trait Sensor: DynClone + Debug {
    fn set_enabled(&mut self, enabled: bool);
    fn enabled(&self) -> bool;
    fn sense(
        &self,
        combatant_isometry: &Isometry3<f32>,
        query_pipeline: &QueryPipeline,
        rigid_body_set: &RigidBodySet,
        collider_set: &ColliderSet,
        active_colliders: &CollidersMapT,
        combatants: &CombatantsMapT,
        balls: &BallsMapT,
        current_tick: GameTickNumber,
    ) -> Vec<ExpiringBelief>;
}

dyn_clone::clone_trait_object!(Sensor);