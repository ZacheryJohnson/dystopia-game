use nalgebra::Quaternion;
use rapier3d::{na::Vector3, prelude::*};
use super::{feature::NavmeshPathingType, ArenaFeature};

#[derive(Clone, Copy, PartialEq)]
pub enum BarrierPathing {
    Disabled,
    Enabled
}

/// All barriers are rectangular prisms.
/// Barriers can be used as walls (which aren't pathable by characters)
/// or as floors (which are pathable by characters)
pub struct ArenaBarrier {
    /// Center point of the barrier
    origin: Vector3<f32>,

    shape: SharedShape,

    /// Quaternion of the rotation
    rotation: Quaternion<f32>,

    /// Is this 
    pathing: BarrierPathing,
}

impl ArenaBarrier {
    pub fn new(
        origin: Vector3<f32>,
        size: Vector3<f32>,
        rotation: Quaternion<f32>,
        pathing: BarrierPathing,
    ) -> ArenaBarrier {
        let shape = SharedShape::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0);

        ArenaBarrier {
            origin,
            shape,
            rotation,
            pathing
        }
    }
}

impl ArenaFeature for ArenaBarrier {
    fn build_rigid_body(&self) -> Option<RigidBody> {
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(self.origin)
            .rotation(self.rotation.vector().into())
            .build();

        Some(rigid_body)
    }

    fn build_collider(&self) -> Option<Collider> {
        let restitution = if self.pathing == BarrierPathing::Disabled {
            0.1
        } else {
            1.0
        };

        let friction = if self.pathing == BarrierPathing::Disabled {
            2.0
        } else {
            1.4
        };

        let collider = ColliderBuilder::new(self.shape.clone())
            .mass(100000.0)
            .restitution(restitution)
            .friction(friction)
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
        if self.pathing == BarrierPathing::Enabled { NavmeshPathingType::Generate } else { NavmeshPathingType::Block }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}