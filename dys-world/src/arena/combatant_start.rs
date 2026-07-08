use rapier3d::na::Quaternion;
use rapier3d::prelude::Vec3;
use super::feature::{ArenaFeature, NavmeshPathingType};

/// Location where a player can be spawned
pub struct ArenaCombatantStart {    
    /// Center point of the player spawn spot
    pub origin: Vec3,

    /// Which team does this spawn point belong to
    pub is_home_team: bool,

    pub rotation: Quaternion<f32>
}

impl ArenaFeature for ArenaCombatantStart {
    fn origin(&self) -> &Vec3 {
        &self.origin
    }

    fn rotation(&self) -> &Quaternion<f32> {
        &self.rotation
    }

    fn pathing_type(&self) -> NavmeshPathingType {
        NavmeshPathingType::Skip
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }    
}