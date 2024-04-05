use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle};

pub trait GameObject {
    fn rigid_body_handle(&self) -> Option<RigidBodyHandle>;

    fn collider_handle(&self) -> Option<ColliderHandle>;
}