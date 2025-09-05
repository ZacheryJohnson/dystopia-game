use std::fmt::{Debug, Formatter};
use nalgebra::Quaternion;
use rapier3d::{na::vector, prelude::*};

use self::{ball_spawn::ArenaBallSpawn, feature::ArenaFeature, plate::ArenaPlate, combatant_start::ArenaCombatantStart, barrier::ArenaBarrier, barrier::BarrierPathing};

pub mod feature;

pub mod barrier;
pub mod plate;
pub mod combatant_start;
pub mod ball_spawn;
pub mod navmesh;
pub(crate) mod serde;

const PLATE_VERTICAL_HEIGHT: f32 = 5.0;

pub struct Arena {
    all_features: Vec<Box<dyn ArenaFeature>>
}

impl Debug for Arena {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Arena <no impl atm>")
    }
}

impl Arena {
    pub fn new_with_testing_defaults() -> Arena {
        Arena {
            // X: west-east; Y: up-down; Z: north-south
            all_features: vec![
                // West Wall
                Box::new(
                    ArenaBarrier::new(vector![0.0, 0.0, 50.0], vector![3.0, 60.0, 100.0], Quaternion::identity(), BarrierPathing::Disabled)
                ),
                // East Wall
                Box::new(
                    ArenaBarrier::new(vector![100.0, 0.0, 50.0], vector![3.0, 60.0, 100.0], Quaternion::identity(), BarrierPathing::Disabled)
                ),
                // South Wall
                Box::new(
                    ArenaBarrier::new(vector![50.0, 0.0, 0.0], vector![100.0, 60.0, 3.0], Quaternion::identity(), BarrierPathing::Disabled)
                ),
                // North Wall
                Box::new(
                    ArenaBarrier::new(vector![50.0, 0.0, 100.0], vector![100.0, 60.0, 3.0], Quaternion::identity(), BarrierPathing::Disabled)
                ),
                // Floor
                Box::new(
                    ArenaBarrier::new(vector![50.0, -5.0, 50.0], vector![100.0, 10.0, 100.0], Quaternion::identity(), BarrierPathing::Enabled)
                ),
                // Plate
                Box::new(
                    ArenaPlate { id: 1, origin: vector![50.0, 0.0, 50.0], shape: SharedShape::cylinder(PLATE_VERTICAL_HEIGHT, 10.0), rotation: Quaternion::identity() }
                ),
                // South Ball Spawn
                Box::new(
                    ArenaBallSpawn { origin: vector![50.0, 0.0, 25.0], rotation: Quaternion::identity() }
                ),
                // North Ball Spawn
                Box::new(
                    ArenaBallSpawn { origin: vector![50.0, 0.0, 75.0], rotation: Quaternion::identity() }
                ),
                // North Plate Wall
                Box::new(
                    ArenaBarrier::new(vector![50.0, 0.0, 70.0], vector![30.0, 10.0, 5.0], Quaternion::identity(), BarrierPathing::Disabled)
                ),
                // South Plate Wall
                Box::new(
                    ArenaBarrier::new(vector![50.0, 0.0, 30.0], vector![30.0, 10.0, 5.0], Quaternion::identity(), BarrierPathing::Disabled)
                ),
                // Home Team Player 1 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 80.0], rotation: Quaternion::identity(), is_home_team: true }
                ),
                // Home Team Player 2 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 65.0], rotation: Quaternion::identity(), is_home_team: true }
                ),
                // Home Team Player 3 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 50.0], rotation: Quaternion::identity(), is_home_team: true }
                ),
                // Home Team Player 4 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 35.0], rotation: Quaternion::identity(), is_home_team: true }
                ),
                // Home Team Player 5 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 20.0], rotation: Quaternion::identity(), is_home_team: true }
                ),
                // Away Team Player 1 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 80.0], rotation: Quaternion::identity(), is_home_team: false }
                ),
                // Away Team Player 2 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 65.0], rotation: Quaternion::identity(), is_home_team: false }
                ),
                // Away Team Player 3 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 50.0], rotation: Quaternion::identity(), is_home_team: false }
                ),
                // Away Team Player 4 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 35.0], rotation: Quaternion::identity(), is_home_team: false }
                ),
                // Away Team Player 5 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 20.0], rotation: Quaternion::identity(), is_home_team: false }
                ),
            ]
        }
    }

    pub fn all_features(&self) -> Vec<&dyn ArenaFeature> {
        self
            .all_features
            .iter()
            .map(|feature| &**feature)
            .collect()
    }

    pub fn features<T: ArenaFeature + 'static>(&self) -> Vec<&T> {
        self
            .all_features
            .iter()
            .filter_map(|feature| feature.as_any().downcast_ref::<T>())
            .collect()
    }
}