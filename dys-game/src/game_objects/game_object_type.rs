use dys_world::arena::plate::PlateId;

use super::{ball::BallId, combatant::CombatantId};

#[derive(Debug)]
pub enum GameObjectType {
    Invalid,
    Wall,
    Ball(BallId),
    Combatant(CombatantId),
    BallSpawn,
    Plate(PlateId),
}