use dys_world::arena::plate::PlateId;
use ordered_float::OrderedFloat;
use crate::game_objects::{ball::BallId, combatant::CombatantId};

/// Beliefs are an agent's understanding of the world.
/// These aren't necessarily true statements about actual game state,
/// but serve as data points for making "rational" decisions.
/// 
/// For example, an agent may believe that an enemy combatant is going
/// to continue running in a straight line, and would use that belief to 
/// aim the ball some distance in front of the runner. 
/// However, the enemy combatant is not affected by or aware of that belief,
/// and may choose to do any action.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(super) enum Belief {
    SelfOnPlate,
    SelfHasBall,
    NearestEnemyCombatant { distance: OrderedFloat<f32>, combatant_id: CombatantId },
    NearestAvailableBall { distance: OrderedFloat<f32>, ball_id: BallId },
    NearestFriendlyCombatant { distance: OrderedFloat<f32>, combatant_id: CombatantId },
    NearestPlate { distance: OrderedFloat<f32>, plate_id: PlateId },
}
