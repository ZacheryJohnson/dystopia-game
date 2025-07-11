use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use dyn_clone::DynClone;
use rapier3d::na::Isometry3;
use crate::ai::belief::ExpiringBelief;
use crate::game_state::GameState;

pub trait Sensor: DynClone + Debug {
    fn set_enabled(&mut self, enabled: bool);
    fn enabled(&self) -> bool;
    fn set_yields_beliefs(&mut self, _yields_beliefs: bool) {}
    fn sense(
        &self,
        combatant_isometry: &Isometry3<f32>,
        game_state: Arc<Mutex<GameState>>,
    ) -> (bool, Vec<ExpiringBelief>);
}

dyn_clone::clone_trait_object!(Sensor);