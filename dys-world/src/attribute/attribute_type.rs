use serde::{Deserialize, Serialize};

/// Each combatant has a collection of attributes which dictate their strengths and weaknesses.
/// Attributes can be provided through limbs or through mental quirks
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum AttributeType {
    /// Overpower opponents.
    Strength,

    /// Swiftness.
    Dexterity,

    /// Mental processing speed.
    Cognition,

    /// Attune with one's body.
    Coordination,

    /// Stand out amongst peers.
    /// Higher values make the combatant prefer self-serving plays to team-oriented ones.
    Ego,

    /// Absorb oncoming blows.
    Constitution,

    /// Stay grounded against adversaries.
    Stability,

    /// Draw the attention of your adversaries.
    Presence,

    /// Ignore distractions and focus on the task at hand.
    /// Higher values give the combatant better resistance to high-presence combatants.
    Stoicism,

    /// Employment takes priority (not that you have a choice).
    Commitment,

    /// Survival at any cost.
    SelfPreservation,

    /// Ability to warn and collaborate.
    Communication,
}