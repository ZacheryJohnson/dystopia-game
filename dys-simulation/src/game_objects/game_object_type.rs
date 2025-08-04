use dys_world::arena::plate::PlateId;
use dys_world::combatant::instance::CombatantInstanceId;
use super::ball::BallId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameObjectType {
    Invalid,
    Barrier,
    Ball(BallId),
    Combatant(CombatantInstanceId),
    BallSpawn,
    Plate(PlateId),
}