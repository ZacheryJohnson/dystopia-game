use bevy::math::Vec3;
use bevy::prelude::Component;
use dys_simulation::game_objects::ball::BallId;
use dys_simulation::game_objects::combatant::CombatantId;
use dys_world::combatant::instance::CombatantInstanceId;

/// All objects in the simulation visualization will have this component.
/// This allows us to easily clean up if the user wants to reload/leave the visualization.
#[derive(Component)]
pub struct VisualizationObject;

#[derive(Component)]
pub struct CombatantVisualizer {
    pub id: CombatantId,
    pub instance_id: CombatantInstanceId,
    pub desired_location: Vec3,
    pub last_position: Vec3,
}

#[derive(Component)]
pub struct BallVisualizer {
    pub id: BallId,
    pub desired_location: Vec3,
    pub last_position: Vec3,
    pub desired_scale: Vec3,
    pub last_scale: Vec3,
    pub desired_charge: f32,
}

#[derive(Component)]
pub struct ExplosionVisualizer {
    pub opacity: u8,
}

#[derive(Component)]
pub struct BarrierVisualizer;

#[derive(Component)]
pub struct PlateVisualizer;

#[derive(Component)]
pub struct FxEntity {
    pub current_lifespan_in_ticks: usize,
}

impl Default for FxEntity {
    fn default() -> Self {
        FxEntity { current_lifespan_in_ticks: 0 }
    }
}