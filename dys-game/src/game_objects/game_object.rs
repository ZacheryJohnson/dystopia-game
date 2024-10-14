use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle};

use crate::game_tick::GameTickNumber;

pub trait GameObject {
    type GameObjectIdT: Eq;
    type GameStateT;

    fn id(&self) -> Self::GameObjectIdT;

    fn rigid_body_handle(&self) -> Option<RigidBodyHandle>;

    fn collider_handle(&self) -> Option<ColliderHandle>;

    /// Updates the state of a game object.
    /// Returns the old state and timestamp when that state began.
    fn change_state(&mut self, current_tick: GameTickNumber, new_state: Self::GameStateT) -> (Self::GameStateT, GameTickNumber);

    fn is_dirty(&self) -> bool;
}