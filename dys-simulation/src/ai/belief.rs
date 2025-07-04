#![allow(non_snake_case)] // this shouldn't be necessary for enums?

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use rapier3d::na::Vector3;
use dys_satisfiable::*;
use dys_satisfiable_macros::{Satisfiable, UniqueKey};
use crate::game_objects::ball::BallId;
use crate::game_objects::combatant::CombatantId;
use crate::game_objects::plate::PlateId;
use crate::game_tick::GameTickNumber;

/// Beliefs are an agent's understanding of the world.
/// These aren't necessarily true statements about actual game state,
/// but serve as data points for making "rational" decisions.
///
/// For example, an agent may believe that an enemy combatant is going
/// to continue running in a straight line, and would use that belief to
/// aim the ball some distance in front of the runner.
/// However, the enemy combatant is not affected by or aware of that belief,
/// and may choose to do any action.
#[derive(Clone, Copy, Debug, PartialEq, Satisfiable, UniqueKey)]
pub enum Belief {
    ScannedEnvironment {
        #[unique]
        tick: GameTickNumber,
    },
    BallPosition {
        #[unique]
        ball_id: BallId,
        position: Vector3<f32>,
        trajectory: Vector3<f32>,
    },
    CombatantPosition {
        #[unique]
        combatant_id: CombatantId,
        position: Vector3<f32>,
    },
    PlatePosition {
        #[unique]
        plate_id: PlateId,
        position: Vector3<f32>,
    },
    OnPlate {
        #[unique]
        plate_id: PlateId,
        #[unique]
        combatant_id: CombatantId,
    },
    HeldBall {
        #[unique]
        ball_id: BallId,
        #[unique]
        combatant_id: CombatantId,
    },
    InBallPickupRange {
        #[unique]
        ball_id: BallId,
        #[unique]
        combatant_id: CombatantId,
    },
    CanReachCombatant {
        #[unique]
        self_combatant_id: CombatantId,
        #[unique]
        target_combatant_id: CombatantId,
    },
    BallThrownAtCombatant {
        #[unique]
        ball_id: BallId,
        #[unique]
        thrower_combatant_id: CombatantId,
        #[unique]
        target_combatant_id: CombatantId,
        target_on_plate: Option<PlateId>,
    },
    BallIsFlying {
        #[unique]
        ball_id: BallId,
    },
    DirectLineOfSightToCombatant {
        #[unique]
        self_combatant_id: CombatantId,
        #[unique]
        other_combatant_id: CombatantId,
    },
    CombatantShoved {
        #[unique]
        combatant_id: CombatantId,
        on_plate: Option<PlateId>,
    },
    BallCaught {
        #[unique]
        combatant_id: CombatantId,
        ball_id: BallId,
    },
}

#[derive(Clone, Debug)]
pub struct ExpiringBelief {
    pub belief: Belief,
    pub expires_on_tick: Option<GameTickNumber>,
}

impl ExpiringBelief {
    pub fn new(belief: Belief, expires_on_tick: Option<GameTickNumber>) -> ExpiringBelief {
        ExpiringBelief {
            belief,
            expires_on_tick,
        }
    }

    pub fn from_beliefs<'a>(
        beliefs: impl IntoIterator<Item = &'a Belief>,
        expires_on_tick: Option<GameTickNumber>,
    ) -> Vec<ExpiringBelief> {
        let mut expiring = vec![];

        for belief in beliefs.into_iter() {
            expiring.push(ExpiringBelief::new(belief.to_owned(), expires_on_tick))
        }

        expiring
    }
}

impl From<ExpiringBelief> for Belief {
    fn from(value: ExpiringBelief) -> Self {
        value.belief
    }
}

/// BeliefSets are collections of beliefs that allow for tests against existing beliefs.
#[derive(Clone, Default, Debug)]
pub struct BeliefSet {
    unsourced_beliefs: Vec<ExpiringBelief>,
    sourced_beliefs: HashMap<u32, Vec<ExpiringBelief>>,
}

impl BeliefSet {
    pub fn empty() -> BeliefSet {
        BeliefSet::from(&[])
    }

    pub fn from(beliefs: &[Belief]) -> BeliefSet {
        BeliefSet {
            unsourced_beliefs: ExpiringBelief::from_beliefs(beliefs, None),
            sourced_beliefs: HashMap::new(),
        }
    }

    pub fn expire_stale_beliefs(&mut self, current_tick: GameTickNumber) {
        let retain_fn = |expiring_belief: &ExpiringBelief| {
            expiring_belief.expires_on_tick.is_none() || current_tick < expiring_belief.expires_on_tick.unwrap()
        };

        self.unsourced_beliefs.retain(retain_fn);

        self.sourced_beliefs.iter_mut().for_each(|(_, expiring_beliefs)| {
            expiring_beliefs.retain(retain_fn);
        });
    }

    fn upsert_unique_unsourced_belief(&mut self, belief: ExpiringBelief) {
        self.unsourced_beliefs.retain(|current_belief| !current_belief.belief.has_same_unique_key(&belief.belief));
        self.unsourced_beliefs.push(belief);
    }

    fn upsert_unique_sourced_belief(&mut self, belief: ExpiringBelief, source_id: u32) {
        match self.sourced_beliefs.entry(source_id) {
            Entry::Occupied(mut entry) => {
                let existing_beliefs = entry.get_mut();
                existing_beliefs.retain(|current_belief| !current_belief.belief.has_same_unique_key(&belief.belief));
                existing_beliefs.push(belief);
            },
            Entry::Vacant(empty) => {
                empty.insert(vec![belief]);
            }
        }
    }

    pub fn add_belief(&mut self, belief: Belief) {
        self.upsert_unique_unsourced_belief(ExpiringBelief::new(belief, None))
    }

    pub fn add_beliefs(&mut self, beliefs: &[Belief]) {
        for belief in beliefs {
            self.add_belief(*belief);
        }
    }

    pub fn add_beliefs_from_source(&mut self, source_id: u32, beliefs: &[Belief]) {
        let expiring_beliefs = beliefs.iter().map(|belief| ExpiringBelief::new(belief.to_owned(), None)).collect::<Vec<_>>();
        self.add_expiring_beliefs_from_source(source_id, &expiring_beliefs)
    }

    pub fn add_expiring_beliefs_from_source(
        &mut self,
        source_id: u32,
        beliefs: &[ExpiringBelief],
    ) {
        for belief in beliefs {
            self.upsert_unique_sourced_belief(
                belief.to_owned(),
                source_id,
            );
        }
    }

    pub fn remove_belief(&mut self, belief: Belief) {
        self.unsourced_beliefs.retain(|b| b.belief != belief)
    }

    pub fn remove_beliefs_by_test(&mut self, belief_test: &(impl SatisfiabilityTest<ConcreteT=Belief> + Debug)) {
        let retain_fn = |expiring_belief: &ExpiringBelief| {
            let belief = &expiring_belief.belief;
            !(belief_test.is_same_variant(belief) && belief_test.satisfied_by(belief.to_owned()))
        };

        self.unsourced_beliefs.retain(retain_fn);
        for beliefs in self.sourced_beliefs.values_mut() {
            beliefs.retain(retain_fn);
        }
    }

    pub fn remove_beliefs_from_source(&mut self, source_id: u32) {
        self.sourced_beliefs.remove(&source_id);
    }

    pub fn beliefs(&self) -> Vec<Belief> {
        let sourced_beliefs = self.sourced_beliefs.values().flatten();
        self
            .unsourced_beliefs
            .clone()
            .iter()
            .chain(sourced_beliefs)
            .map(|belief| belief.belief.to_owned())
            .collect::<Vec<Belief>>()
    }

    pub fn sourced_beliefs(&self) -> HashMap<u32, Vec<ExpiringBelief>> {
        let mut sourced_expiring_beliefs = self.sourced_beliefs.clone();
        sourced_expiring_beliefs.insert(0, self.unsourced_beliefs.clone());
        sourced_expiring_beliefs
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
    pub fn can_satisfy(&self, satisfiable: &(impl SatisfiabilityTest<ConcreteT=Belief> + Debug + Clone)) -> bool {
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

        assert!(belief_set.can_satisfy(&satisfiable));
    }

    #[test]
    fn test_different_type_doesnt_satisfy() {
        let belief_set = BeliefSet::from(&vec![Belief::BallPosition {
            ball_id: 1,
            position: Vector3::<f32>::identity(),
            trajectory: Vector3::<f32>::identity(),
        }]);

        let satisfiable = SatisfiableBelief::OnPlate();

        assert!(!belief_set.can_satisfy(&satisfiable));
    }

    #[test]
    fn test_same_type_exact_plate_id_match_satisfies() {
        let belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);

        let satisfiable = SatisfiableBelief::OnPlate()
            .plate_id(SatisfiableField::Exactly(1));

        assert!(belief_set.can_satisfy(&satisfiable));
    }

    #[test]
    fn test_same_type_exact_combatant_id_match_satisfies() {
        let belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);

        let satisfiable = SatisfiableBelief::OnPlate()
            .combatant_id(SatisfiableField::Exactly(1));

        assert!(belief_set.can_satisfy(&satisfiable));
    }

    #[test]
    fn test_same_type_all_fields_match_satisfies() {
        let belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);

        let satisfiable = SatisfiableBelief::OnPlate()
            .plate_id(SatisfiableField::Exactly(1))
            .combatant_id(SatisfiableField::Exactly(1));

        assert!(belief_set.can_satisfy(&satisfiable));
    }

    #[test]
    fn test_same_type_mismatch_doesnt_satisfy() {
        let belief_set = BeliefSet::from(&vec![Belief::OnPlate { plate_id: 1, combatant_id: 1 }]);

        let satisfiable = SatisfiableBelief::OnPlate()
            .plate_id(SatisfiableField::Exactly(2))
            .combatant_id(SatisfiableField::Exactly(1));

        assert!(!belief_set.can_satisfy(&satisfiable));
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

    #[test]
    fn consumed_beliefs_correctly_removed() {
        let mut belief_set = BeliefSet::from(&vec![
            Belief::HeldBall { ball_id: 1, combatant_id: 1 },
        ]);

        assert_eq!(belief_set.beliefs().len(), 1);

        belief_set.remove_beliefs_by_test(&SatisfiableBelief::HeldBall()
            .ball_id(SatisfiableField::Exactly(1))
            .combatant_id(SatisfiableField::Exactly(1))
        );

        assert_eq!(belief_set.beliefs().len(), 0);
    }
}