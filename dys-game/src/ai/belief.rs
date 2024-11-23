use std::mem;

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
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Belief {
    SelfOnPlate,
    SelfHasBall,
    SelfCanReachBall { ball_id: BallId, },
    BallNotHeld { ball_id: BallId },
    NearestEnemyCombatant { distance: OrderedFloat<f32>, combatant_id: CombatantId },
    NearestAvailableBall { distance: OrderedFloat<f32>, ball_id: BallId },
    NearestFriendlyCombatant { distance: OrderedFloat<f32>, combatant_id: CombatantId },
    NearestPlate { distance: OrderedFloat<f32>, plate_id: PlateId },
}

impl Belief {
    pub fn is_a(&self, desired_belief: &Belief) -> bool {
        mem::discriminant(self) == mem::discriminant(desired_belief)
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::Belief;

    #[test]
    fn test_is_a() {
        let belief_original = Belief::NearestPlate { distance: OrderedFloat::from(1.0), plate_id: 1 };
        let belief_same_type = Belief::NearestPlate { distance: OrderedFloat::from(3.0), plate_id: 2 };
         
        assert!(belief_same_type.is_a(&belief_original));

        let belief_different_type = Belief::SelfOnPlate;
        
        assert!(!belief_different_type.is_a(&belief_original));
    }
}
