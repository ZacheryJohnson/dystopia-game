use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle};

use super::game_object::GameObject;

pub type PlateId = u8;

pub struct PlateObject {
    id: PlateId,
    collider_handle: ColliderHandle,
}

impl PlateObject {
    pub fn new(id: PlateId, collider_handle: ColliderHandle) -> PlateObject {
        PlateObject {
            id,
            collider_handle
        }
    }
}

impl GameObject for PlateObject {
    type GameObjectIdT = PlateId;
    type GameStateT = ();

    fn id(&self) -> Self::GameObjectIdT {
        self.id
    }

    fn rigid_body_handle(&self) -> Option<RigidBodyHandle> {
        None
    }

    fn collider_handle(&self) -> Option<ColliderHandle> {
        Some(self.collider_handle)
    }

    fn change_state(&mut self, _current_tick: crate::game_tick::GameTickNumber, _new_state: Self::GameStateT) -> (Self::GameStateT, crate::game_tick::GameTickNumber) {
        panic!("plates cannot change state")
    }

    fn is_dirty(&self) -> bool {
        panic!("plates cannot be dirty")
    }
}