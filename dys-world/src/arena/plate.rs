use rapier3d::na::Vector3;
use rapier3d::prelude::*;

use super::ArenaFeature;

pub enum ArenaPlateShape {
    Circle { radius: f32 },
    // Rect { width: f32, height: f32 },
    // /// Equilateral triangle
    // Triangle { width: f32, height: f32 },
}

pub struct ArenaPlate {
    /// Center point of the wall
    pub origin: Vector3<f32>,

    /// Shape of the plate, including size of that shape
    pub shape: ArenaPlateShape,

    /// Quaternion of the rotation
    pub rotation: Vector3<f32>,
}

impl ArenaFeature for ArenaPlate {
    fn build_rigid_body(&self) -> Option<RigidBody> {
        // Plates do not need rigid bodies (at least with current design)
        // Maybe there's a world where plates move around but meh, feature creep
        None
    }

    fn build_collider(&self) -> Option<Collider> {
        const PLATE_VERTICAL_HEIGHT: f32 = 100.0;

        let shape: SharedShape = match &self.shape {
            ArenaPlateShape::Circle { radius } => SharedShape::cylinder(PLATE_VERTICAL_HEIGHT, *radius),
            // ArenaPlateShape::Rect { width, height } => SharedShape::cuboid(*width, *height, PLATE_VERTICAL_HEIGHT),
            // ArenaPlateShape::Triangle { width, height } => SharedShape::triangle(point![0.0, 0.0, 0.0], point![*width, 0.0, 0.0], point![*width / 2.0, *height, 0.0]), // TODO: pretty sure this is broken: 0 height on this collider
        };

        let collider = ColliderBuilder::new(shape)
            .translation(self.origin)
            .rotation(self.rotation)
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