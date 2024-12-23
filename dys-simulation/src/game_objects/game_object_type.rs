use dys_world::arena::plate::PlateId;

use super::{ball::BallId, combatant::CombatantId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameObjectType {
    Invalid,
    Barrier,
    Ball(BallId),
    Combatant(CombatantId),
    BallSpawn,
    Plate(PlateId),
}