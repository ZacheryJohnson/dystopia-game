use std::any::Any;

use rapier3d::{dynamics::RigidBody, geometry::{Collider, SharedShape}, na::{Quaternion, Vector3}};

pub trait ArenaFeature {    
    fn build_rigid_body(&self) -> Option<RigidBody> { None }

    fn build_collider(&self) -> Option<Collider> { None }

    fn origin(&self) -> &Vector3<f32>;

    fn rotation(&self) -> &Quaternion<f32>;

    fn shape(&self) -> Option<&SharedShape> { None }

    fn is_pathable(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
}