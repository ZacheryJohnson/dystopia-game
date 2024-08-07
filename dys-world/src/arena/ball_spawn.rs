use rapier3d::na::{Quaternion, Vector3};

use super::feature::{ArenaFeature, NavmeshPathingType};

/// Location where a ball can be spawned
pub struct ArenaBallSpawn {    
    /// Center point of the ball spawn spot
    pub origin: Vector3<f32>,

    pub rotation: Quaternion<f32>
}

impl ArenaFeature for ArenaBallSpawn {
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