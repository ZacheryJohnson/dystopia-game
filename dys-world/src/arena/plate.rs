use nalgebra::Quaternion;
use rapier3d::na::Vector3;
use rapier3d::prelude::*;

use super::feature::NavmeshPathingType;
use super::ArenaFeature;

pub type PlateId = u8;

pub struct ArenaPlate {
    pub id: PlateId,

    /// Center point of the plate
    pub origin: Vector3<f32>,

    /// Shape of the plate, including size of that shape
    pub shape: SharedShape,

    /// Quaternion of the rotation
    pub rotation: Quaternion<f32>,
}

impl ArenaFeature for ArenaPlate {
    fn build_rigid_body(&self) -> Option<RigidBody> {
        // Plates do not need rigid bodies (at least with current design)
        // Maybe there's a world where plates move around but meh, feature creep
        None
    }

    fn build_collider(&self) -> Option<Collider> {
        // Plates don't have a physical height, but this is to accomodate a tall vertical collider to detect collisions.
        // This should be large enough to accommodate different player sizes and collider differences,
        // but not so large that players flying through the air are counted towards plate progress.

        let collider = ColliderBuilder::new(self.shape.clone())
            .translation(self.origin)
            .rotation(self.rotation.vector().into())
            .sensor(true)
            .build();

        Some(collider)
    }

    fn origin(&self) -> &Vector3<f32> {
        &self.origin
    }

    fn rotation(&self) -> &Quaternion<f32> {
        &self.rotation
    }

    fn shape(&self) -> Option<&SharedShape> {
        Some(&self.shape)
    }

    fn pathing_type(&self) -> NavmeshPathingType {
        NavmeshPathingType::Skip
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}