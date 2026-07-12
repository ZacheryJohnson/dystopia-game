use bevy::prelude::*;
use dys_world::combatant::instance::CombatantInstanceId;

/// Displays the combatant's name underneath them as they move around the arena.
#[derive(Component)]
pub struct CombatantIdText {
    pub combatant_id: CombatantInstanceId,
    pub is_stunned: bool,
}