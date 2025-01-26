use std::collections::hash_map::Entry;
use std::collections::HashMap;
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
    BallThrownAtCombatant { ball_id: BallId, thrower_id: CombatantId, target_id: CombatantId },
}

/// BeliefSets are collections of beliefs that allow for tests against existing beliefs.
#[derive(Clone, Default, Debug)]
pub struct BeliefSet {
    unsourced_beliefs: Vec<Belief>,
    sourced_beliefs: HashMap<u32, Vec<Belief>>,
}

impl BeliefSet {
    pub fn empty() -> BeliefSet {
        BeliefSet::from(&vec![])
    }

    pub fn from(beliefs: &[Belief]) -> BeliefSet {
        BeliefSet {
            unsourced_beliefs: beliefs.to_vec(),
            sourced_beliefs: HashMap::new(),
        }
    }

    pub fn add_belief(&mut self, belief: Belief) {
        self.unsourced_beliefs.push(belief)
    }

    pub fn add_beliefs(&mut self, beliefs: &[Belief]) {
        for belief in beliefs {
            self.add_belief(*belief);
        }
    }

    pub fn add_beliefs_from_source(&mut self, source_id: u32, beliefs: &[Belief]) {
        if beliefs.is_empty() {
            return;
        }

        match self.sourced_beliefs.entry(source_id) {
            Entry::Occupied(mut entry) => {
                for belief in beliefs {
                    entry.get_mut().push(*belief);
                }
            },
            Entry::Vacant(mut empty) => {
                empty.insert(beliefs.to_vec());
            }
        }
    }

    pub fn remove_belief(&mut self, belief: Belief) {
        self.unsourced_beliefs.retain(|b| b != &belief)
    }

    pub fn remove_beliefs_from_source(&mut self, source_id: u32,) {
        self.sourced_beliefs.remove(&source_id);
    }

    fn beliefs(&self) -> Vec<Belief> {
        let sourced_beliefs = self.sourced_beliefs.values().flatten();
        self
            .unsourced_beliefs
            .clone()
            .iter()
            .chain(sourced_beliefs)
            .map(|belief| belief.to_owned())
            .collect()
    }

    #[tracing::instrument(
        name = "belief::can_satisfy",
        skip_all,
        fields(belief = tracing::field::debug(satisfiable.clone()))
        level = "trace"
    )]
    /// Can any beliefs in this belief set with the same variant as the satisfiable belief be satisfied?
    /// Beliefs of different variants are ignored.
    /// If no beliefs of the satisfiable belief's variant exist in the belief set,
    /// returns false.
    pub fn can_satisfy(&self, satisfiable: impl SatisfiabilityTest<ConcreteT=Belief> + Debug + Clone) -> bool {
        self
            .beliefs()
            .iter()
            .filter(|belief| satisfiable.is_same_variant(belief))
            .any(|belief| satisfiable.satisfied_by(*belief))
    }

    #[tracing::instrument(
        name = "belief::all_satisfy",
        skip_all,
        fields(belief = tracing::field::debug(satisfiable.clone()))
        level = "trace"
    )]
    /// Can all beliefs in this belief set with the same variant as the satisfiable belief be satisfied?
    /// Beliefs of different variants are ignored.
    /// If no beliefs of the satisfiable belief's variant exist in the belief set,
    /// returns true.
    pub fn all_satisfy(&self, satisfiable: impl SatisfiabilityTest<ConcreteT=Belief> + Debug + Clone) -> bool {
        self
            .beliefs()
            .iter()
            .filter(|belief| satisfiable.is_same_variant(belief))
            .all(|b| satisfiable.satisfied_by(*b))
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

    #[test]
    fn all_satisfy_same_plate_id() {
        let belief_set = BeliefSet::from(&vec![
            Belief::OnPlate { plate_id: 1, combatant_id: 1 },
            Belief::OnPlate { plate_id: 1, combatant_id: 2 },
            Belief::OnPlate { plate_id: 1, combatant_id: 3 },
            Belief::OnPlate { plate_id: 1, combatant_id: 4 },
            Belief::OnPlate { plate_id: 1, combatant_id: 5 }
        ]);

        assert!(belief_set.all_satisfy(
            SatisfiableBelief::OnPlate()
                .plate_id(SatisfiableField::Exactly(1)),
        ));
    }

    #[test]
    fn all_satisfy_none_combatant_6() {
        let belief_set = BeliefSet::from(&vec![
            Belief::OnPlate { plate_id: 1, combatant_id: 1 },
            Belief::OnPlate { plate_id: 1, combatant_id: 2 },
            Belief::OnPlate { plate_id: 1, combatant_id: 3 },
            Belief::OnPlate { plate_id: 1, combatant_id: 4 },
            Belief::OnPlate { plate_id: 1, combatant_id: 5 }
        ]);

        assert!(belief_set.all_satisfy(
            SatisfiableBelief::OnPlate()
                .combatant_id(SatisfiableField::NotExactly(6)),
        ));
    }

    #[test]
    fn all_satisfy_mixed_beliefs() {
        let belief_set = BeliefSet::from(&vec![
            Belief::OnPlate { plate_id: 1, combatant_id: 1 },
            Belief::OnPlate { plate_id: 1, combatant_id: 2 },
            Belief::HeldBall { ball_id: 1, combatant_id: 1 },
        ]);

        assert!(belief_set.all_satisfy(
            SatisfiableBelief::OnPlate()
                .plate_id(SatisfiableField::Exactly(1)),
        ));

        assert!(belief_set.all_satisfy(
            SatisfiableBelief::OnPlate()
                .combatant_id(SatisfiableField::In(vec![1, 2]))
        ));

        assert!(belief_set.all_satisfy(
            SatisfiableBelief::HeldBall()
                .combatant_id(SatisfiableField::NotIn(vec![2, 3]))
        ));
    }

    #[test]
    fn sourced_beliefs_returned_with_unsourced() {
        let mut belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);
        belief_set.add_beliefs_from_source(1, &vec![Belief::HeldBall { ball_id: 1, combatant_id: 1 }]);

        let beliefs = belief_set.beliefs();
        assert_eq!(beliefs.len(), 2);
    }
}