#![allow(non_snake_case)] // this shouldn't be necessary for enums?

use std::fmt::Debug;
use rapier3d::na::Vector3;
use serde::{Deserialize, Serialize};
use dys_satisfiable::*;
use dys_satisfiable_macros::{Satisfiable, UniqueKey};
use dys_world::combatant::instance::CombatantInstanceId;
use crate::game_objects::ball::BallId;
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
#[derive(Clone, Copy, Debug, PartialEq, Satisfiable, UniqueKey, Serialize, Deserialize)]
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
        combatant_id: CombatantInstanceId,
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
        combatant_id: CombatantInstanceId,
    },
    HeldBall {
        #[unique]
        ball_id: BallId,
        #[unique]
        combatant_id: CombatantInstanceId,
    },
    InBallPickupRange {
        #[unique]
        ball_id: BallId,
        #[unique]
        combatant_id: CombatantInstanceId,
    },
    CanReachCombatant {
        #[unique]
        self_combatant_id: CombatantInstanceId,
        #[unique]
        target_combatant_id: CombatantInstanceId,
    },
    BallThrownAtCombatant {
        #[unique]
        ball_id: BallId,
        #[unique]
        thrower_combatant_id: CombatantInstanceId,
        #[unique]
        target_combatant_id: CombatantInstanceId,
        target_on_plate: Option<PlateId>,
    },
    BallIsFlying {
        #[unique]
        ball_id: BallId,
    },
    DirectLineOfSightToCombatant {
        #[unique]
        self_combatant_id: CombatantInstanceId,
        #[unique]
        other_combatant_id: CombatantInstanceId,
    },
    CombatantShoved {
        #[unique]
        combatant_id: CombatantInstanceId,
        on_plate: Option<PlateId>,
    },
    BallCaught {
        #[unique]
        combatant_id: CombatantInstanceId,
        thrower_id: CombatantInstanceId,
        ball_id: BallId,
    },
    CombatantIsStunned {
        #[unique]
        combatant_id: CombatantInstanceId,
    }
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
