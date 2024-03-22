use rapier3d::{dynamics::{RigidBody, RigidBodyBuilder, RigidBodySet}, geometry::{Collider, ColliderBuilder, ColliderSet, SharedShape}, na::{vector, Vector3}, prelude::*};

pub struct Arena {
    features: Vec<Box<dyn ArenaFeature>>
}

impl Arena {
    pub fn new_with_testing_defaults() -> Arena {
        Arena {
            features: vec![
                Box::new(
                    ArenaWall { origin: vector![0.0, 50.0, 0.0], size: vector![1.0, 100.0, 10.0], rotation: vector![0.0, 0.0, 0.0] }
                ),
                Box::new(
                    ArenaWall { origin: vector![100.0, 50.0, 0.0], size: vector![1.0, 100.0, 10.0], rotation: vector![0.0, 0.0, 0.0] }
                ),
                Box::new(
                    ArenaWall { origin: vector![50.0, 0.0, 0.0], size: vector![100.0, 1.0, 10.0], rotation: vector![0.0, 0.0, 0.0] }
                ),
                Box::new(
                    ArenaWall { origin: vector![50.0, 100.0, 0.0], size: vector![100.0, 1.0, 10.0], rotation: vector![0.0, 0.0, 0.0] }
                ),
                Box::new(
                    ArenaPlate { origin: vector![50.0, 50.0, 0.0], shape: ArenaPlateShape::Circle { radius: 10.0 }, rotation: vector![0.0, 0.0, 0.0] }
                )
            ]
        }
    }

    pub fn register_features_physics(&self, mut rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet) {
        for feature in &self.features {
            if let Some(rigid_body) = feature.build_rigid_body() {
                let rigid_body_handle = rigid_body_set.insert(rigid_body);
                if let Some(collider) = feature.build_collider() {
                    let _collider_handle = collider_set.insert_with_parent(collider, rigid_body_handle, &mut rigid_body_set);
                }
            } else {
                if let Some(collider) = feature.build_collider() {
                    let _collider_handle = collider_set.insert(collider);
                }
            }
        }
    }
}

trait ArenaFeature {
    fn build_rigid_body(&self) -> Option<RigidBody>;

    fn build_collider(&self) -> Option<Collider>;
}

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
        let collider = ColliderBuilder::cuboid(self.size.x, self.size.y, self.size.z)
            .translation(self.origin)
            .rotation(self.rotation)
            .build();

        Some(collider)
    }
}

pub enum ArenaPlateShape {
    Circle { radius: f32 },
    Rect { width: f32, height: f32 },
    /// Equilateral triangle
    Triangle { width: f32, height: f32 },
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
            ArenaPlateShape::Rect { width, height } => SharedShape::cuboid(*width, *height, PLATE_VERTICAL_HEIGHT),
            ArenaPlateShape::Triangle { width, height } => SharedShape::triangle(point![0.0, 0.0, 0.0], point![*width, 0.0, 0.0], point![*width / 2.0, *height, 0.0]), // TODO: pretty sure this is broken: 0 height on this collider
        };

        let collider = ColliderBuilder::new(shape)
            .translation(self.origin)
            .rotation(self.rotation)
            .build();

        Some(collider)
    }
}