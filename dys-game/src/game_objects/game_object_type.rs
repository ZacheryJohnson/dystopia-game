use super::{ball::BallId, combatant::CombatantId};

#[derive(Debug)]
pub enum GameObjectType {
    Invalid,
    Wall,
    Ball(BallId),
    Combatant(CombatantId),
}