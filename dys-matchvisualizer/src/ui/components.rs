use bevy::prelude::Component;
use dys_world::combatant::instance::CombatantInstanceId;

/// Shows simulation performance statistics in the visualization.
/// Intended to be dev-only to understand simulation perf at a glance (particularly in web/WASM).
#[derive(Component)]
pub struct GameLogPerfText;

/// Displays the current game time.
#[derive(Component)]
pub struct MatchTimerText;

/// Displays the home team's score.
#[derive(Component)]
pub struct HomeTeamScoreText;

/// Displays how the home team's score has been updated recently.
#[derive(Component)]
pub struct HomeTeamScoreUpdateText;

/// Displays the away team's score.
#[derive(Component)]
pub struct AwayTeamScoreText;

/// Displays how the away team's score has been updated recently.
#[derive(Component)]
pub struct AwayTeamScoreUpdateText;

/// A table showing each combatant's statlines in the visualized match.
#[derive(Component)]
pub struct PostgameScoreboard;

/// Displays the combatant's name underneath them as they move around the arena.
#[derive(Component)]
pub struct CombatantIdText {
    pub combatant_id: CombatantInstanceId,
    pub is_stunned: bool,
}