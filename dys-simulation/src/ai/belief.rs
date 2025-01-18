use std::fmt::Debug;
use rapier3d::na::Vector3;
use dys_satisfiable::*;
use dys_satisfiable_macros::{Satisfiable};
use crate::game_objects::ball::BallId;
use crate::game_objects::combatant::CombatantId;
use crate::game_objects::plate::PlateId;

/// Beliefs are an agent's understanding of the world.
/// These aren't necessarily true statements about actual game state,
/// but serve as data points for making "rational" decisions.
///
/// For example, an agent may believe that an enemy combatant is going
/// to continue running in a straight line, and would use that belief to
/// aim the ball some distance in front of the runner.
/// However, the enemy combatant is not affected by or aware of that belief,
/// and may choose to do any action.
#[derive(Clone, Copy, Debug, PartialEq, Satisfiable)]
pub enum Belief {
    BallPosition { ball_id: BallId, position: Vector3<f32> },
    CombatantPosition { combatant_id: CombatantId, position: Vector3<f32> },
    PlatePosition { plate_id: PlateId, position: Vector3<f32> },
    OnPlate { plate_id: PlateId, combatant_id: CombatantId },
    HeldBall { ball_id: BallId, combatant_id: CombatantId },
    InBallPickupRange { ball_id: BallId, combatant_id: CombatantId },
}

/// BeliefSets are collections of beliefs that allow for tests against existing beliefs.
#[derive(Clone, Default, Debug)]
pub struct BeliefSet {
    beliefs: Vec<Belief>,
}

impl BeliefSet {
    pub fn empty() -> BeliefSet {
        BeliefSet::from(&vec![])
    }

    pub fn from(beliefs: &[Belief]) -> BeliefSet {
        BeliefSet {
            beliefs: beliefs.to_vec(),
        }
    }

    pub fn add_belief(&mut self, belief: Belief) {
        self.beliefs.push(belief)
    }

    pub fn add_beliefs(&mut self, beliefs: &[Belief]) {
        for belief in beliefs {
            self.add_belief(*belief);
        }
    }

    pub fn remove_belief(&mut self, belief: Belief) {
        self.beliefs.retain(|b| b != &belief)
    }

    #[tracing::instrument(
        name = "belief::can_satisfy",
        skip_all,
        fields(belief = tracing::field::debug(satisfiable.clone()))
        level = "trace"
    )]
    pub fn can_satisfy(&self, satisfiable: impl SatisfiabilityTest<ConcreteT=Belief> + Debug + Clone) -> bool {
        self
            .beliefs
            .iter()
            .any(|b| satisfiable.satisfied_by(*b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_type_all_ignored_satisfies() {
        let belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);

        let satisfiable = SatisfiableBelief::OnPlate();

        assert!(belief_set.can_satisfy(satisfiable));
    }

    #[test]
    fn test_different_type_doesnt_satisfy() {
        let belief_set = BeliefSet::from(&vec![Belief::BallPosition { ball_id: 1, position: Vector3::<f32>::identity() }]);

        let satisfiable = SatisfiableBelief::OnPlate();

        assert!(!belief_set.can_satisfy(satisfiable));
    }

    #[test]
    fn test_same_type_exact_plate_id_match_satisfies() {
        let belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);

        let satisfiable = SatisfiableBelief::OnPlate()
            .plate_id(SatisfiableField::Exactly(1));

        assert!(belief_set.can_satisfy(satisfiable));
    }

    #[test]
    fn test_same_type_exact_combatant_id_match_satisfies() {
        let belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);

        let satisfiable = SatisfiableBelief::OnPlate()
            .combatant_id(SatisfiableField::Exactly(1));

        assert!(belief_set.can_satisfy(satisfiable));
    }

    #[test]
    fn test_same_type_all_fields_match_satisfies() {
        let belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);

        let satisfiable = SatisfiableBelief::OnPlate()
            .plate_id(SatisfiableField::Exactly(1))
            .combatant_id(SatisfiableField::Exactly(1));

        assert!(belief_set.can_satisfy(satisfiable));
    }

    #[test]
    fn test_same_type_mismatch_doesnt_satisfy() {
        let belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);

        let satisfiable = SatisfiableBelief::OnPlate()
            .plate_id(SatisfiableField::Exactly(2))
            .combatant_id(SatisfiableField::Exactly(1));

        assert!(!belief_set.can_satisfy(satisfiable));
    }
}