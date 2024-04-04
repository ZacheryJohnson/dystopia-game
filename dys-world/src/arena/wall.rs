use rapier3d::{na::Vector3, prelude::*};

use super::ArenaFeature;

/// All walls are rectangular prisms
pub struct ArenaWall {
    /// Center point of the wall
    pub origin: Vector3<f32>,

    /// XYZ size of the wall
    pub size: Vector3<f32>,

    /// Quaternion of the rotation
    pub rotation: Vector3<f32>,
}

impl ArenaFeature for ArenaWall {
    fn build_rigid_body(&self) -> Option<RigidBody> {
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(self.origin)
            .rotation(self.rotation)
            .build();

        Some(rigid_body)
    }

    fn build_collider(&self) -> Option<Collider> {
        let collider = ColliderBuilder::cuboid(self.size.x / 2.0, self.size.y / 2.0, self.size.z / 2.0)
            .build();

        Some(collider)
    }

    fn origin(&self) -> &Vector3<f32> {
        &self.origin
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}