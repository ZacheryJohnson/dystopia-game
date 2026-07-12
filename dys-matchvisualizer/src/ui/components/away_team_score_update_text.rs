use bevy::prelude::*;

const FONT_FILE: &str = "fonts/Teko-Medium.ttf";

/// Displays how the away team's score has been updated recently.
#[derive(Component)]
pub struct AwayTeamScoreUpdateText;

impl AwayTeamScoreUpdateText {
    pub fn bundle(asset_server: &Res<AssetServer>) -> impl Bundle {
        (
            AwayTeamScoreUpdateText,
            Text(String::from("")),
            TextColor(Color::WHITE),
            TextFont {
                font: FontSource::Handle(asset_server.load(FONT_FILE)),
                font_size: FontSize::VMin(4.0),
                ..default()
            },
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                align_self: AlignSelf::Center,
                top: Val::Vh(2.0),
                left: Val::Vw(70.0),
                ..default()
            },
        )
    }
}