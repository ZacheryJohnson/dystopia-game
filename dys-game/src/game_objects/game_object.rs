use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle, na::Vector3};

pub trait GameObject {
    fn rigid_body_handle(&self) -> Option<RigidBodyHandle>;

    fn collider_handle(&self) -> Option<ColliderHandle>;
}