use std::any::Any;

use rapier3d::{dynamics::RigidBody, geometry::{Collider, SharedShape}, na::{Quaternion, Vector3}};

#[derive(PartialEq, Eq)]
pub enum NavmeshPathingType {
    /// Generate new navmesh nodes to allow pathing over this object
    Generate,
    /// Skip generating new navmesh nodes for this object, but don't block/remove existing nodes
    Skip,
    /// Block any movement over this node, even if others may have generated nodes in this location
    Block
}

pub trait ArenaFeature {    
    fn build_rigid_body(&self) -> Option<RigidBody> { None }

    fn build_collider(&self) -> Option<Collider> { None }

    fn origin(&self) -> &Vector3<f32>;

    fn rotation(&self) -> &Quaternion<f32>;

    fn shape(&self) -> Option<&SharedShape> { None }

    fn pathing_type(&self) -> NavmeshPathingType;

    fn as_any(&self) -> &dyn Any;
}