use rapier3d::na::{Quaternion, Vector3};

use super::feature::{ArenaFeature, NavmeshPathingType};

/// Location where a player can be spawned
pub struct ArenaCombatantStart {    
    /// Center point of the player spawn spot
    pub origin: Vector3<f32>,

    /// Which team does this spawn point belong to
    pub is_home_team: bool,

    pub rotation: Quaternion<f32>
}

impl ArenaFeature for ArenaCombatantStart {
    fn origin(&self) -> &Vector3<f32> {
        &self.origin
    }

    fn pathing_type(&self) -> NavmeshPathingType {
        NavmeshPathingType::Skip
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn rotation(&self) -> &Quaternion<f32> {
        &self.rotation
    }    
}