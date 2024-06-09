use rapier3d::na::Vector3;

use super::feature::ArenaFeature;

/// Location where a player can be spawned
pub struct ArenaCombatantStart {    
    /// Center point of the player spawn spot
    pub origin: Vector3<f32>,

    /// Which team does this spawn point belong to
    pub is_home_team: bool,
}

impl ArenaFeature for ArenaCombatantStart {
    fn origin(&self) -> &Vector3<f32> {
        &self.origin
    }

    fn is_pathable(&self) -> bool {
        true
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}