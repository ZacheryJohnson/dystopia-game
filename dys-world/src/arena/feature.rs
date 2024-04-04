use std::any::Any;

use rapier3d::{dynamics::RigidBody, geometry::Collider, na::Vector3};

pub trait ArenaFeature {
    fn build_rigid_body(&self) -> Option<RigidBody> { None }

    fn build_collider(&self) -> Option<Collider> { None }

    fn origin(&self) -> &Vector3<f32>;

    fn as_any(&self) -> &dyn Any;
}