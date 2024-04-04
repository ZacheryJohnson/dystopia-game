use std::any::{Any, TypeId};

use rapier3d::{dynamics::RigidBodySet, geometry::ColliderSet, na::{vector, Vector3}, prelude::*};

use self::{ball_spawn::ArenaBallSpawn, feature::ArenaFeature, plate::{ArenaPlate, ArenaPlateShape}, combatant_start::ArenaCombatantStart, wall::ArenaWall};

pub mod feature;

mod wall;
mod plate;
mod combatant_start;
mod ball_spawn;

pub struct Arena {
    features: Vec<Box<dyn ArenaFeature>>
}

impl Arena {
    pub fn new_with_testing_defaults() -> Arena {
        Arena {
            // X: horizontal; Y: vertical; Z: depth
            features: vec![
                // West Wall
                Box::new(
                    ArenaWall { origin: vector![0.0, 0.0, 50.0], size: vector![3.0, 10.0, 100.0], rotation: vector![0.0, 0.0, 0.0] }
                ),
                // East Wall
                Box::new(
                    ArenaWall { origin: vector![100.0, 0.0, 50.0], size: vector![3.0, 10.0, 100.0], rotation: vector![0.0, 0.0, 0.0] }
                ),
                // South Wall
                Box::new(
                    ArenaWall { origin: vector![50.0, 0.0, 0.0], size: vector![100.0, 10.0, 3.0], rotation: vector![0.0, 0.0, 0.0] }
                ),
                // North Wall
                Box::new(
                    ArenaWall { origin: vector![50.0, 0.0, 100.0], size: vector![100.0, 10.0, 3.0], rotation: vector![0.0, 0.0, 0.0] }
                ),
                // Floor
                Box::new(
                    ArenaWall { origin: vector![50.0, -5.0, 50.0], size: vector![100.0, 10.0, 100.0], rotation: vector![0.0, 0.0, 0.0] }
                ),
                // Plate
                Box::new(
                    ArenaPlate { origin: vector![50.0, 0.0, 50.0], shape: ArenaPlateShape::Circle { radius: 10.0 }, rotation: vector![0.0, 0.0, 0.0] }
                ),
                // South Ball Spawn
                Box::new(
                    ArenaBallSpawn { origin: vector![50.0, 0.0, 25.0] }
                ),
                // North Ball Spawn
                Box::new(
                    ArenaBallSpawn { origin: vector![50.0, 0.0, 75.0] }
                ),
                // Home Team Player 1 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 80.0], is_home_team: true }
                ),
                // Home Team Player 2 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 65.0], is_home_team: true }
                ),
                // Home Team Player 3 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 50.0], is_home_team: true }
                ),
                // Home Team Player 4 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 35.0], is_home_team: true }
                ),
                // Home Team Player 5 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![15.0, 0.0, 20.0], is_home_team: true }
                ),
                // Away Team Player 1 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 80.0], is_home_team: false }
                ),
                // Away Team Player 2 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 65.0], is_home_team: false }
                ),
                // Away Team Player 3 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 50.0], is_home_team: false }
                ),
                // Away Team Player 4 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 35.0], is_home_team: false }
                ),
                // Away Team Player 5 Start
                Box::new(
                    ArenaCombatantStart { origin: vector![85.0, 0.0, 20.0], is_home_team: false }
                ),
            ]
        }
    }

    pub fn register_features_physics(&self, rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet) {
        for feature in &self.features {
            if let Some(rigid_body) = feature.build_rigid_body() {
                let rigid_body_handle = rigid_body_set.insert(rigid_body);
                if let Some(collider) = feature.build_collider() {
                    let _collider_handle = collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);
                }
            } else {
                if let Some(collider) = feature.build_collider() {
                    let _collider_handle = collider_set.insert(collider);
                }
            }
        }
    }

    pub fn ball_spawns(&self) -> Vec<&ArenaBallSpawn> {
        let spawns = self
            .features
            .iter()
            .filter(|feature| TypeId::of::<ArenaBallSpawn>() == (*feature).type_id())
            .map(|feature| feature.as_any().downcast_ref::<ArenaBallSpawn>().expect("failed to cast feature to a ball spawn"))
            .collect();

        spawns
    }

    pub fn combatant_starts(&self) -> Vec<&ArenaCombatantStart> {
        let starts = self
            .features
            .iter()
            .filter_map(|feature| feature.as_any().downcast_ref::<ArenaCombatantStart>())
            .collect();

        starts
    }
}