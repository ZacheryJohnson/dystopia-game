use bevy::prelude::*;

const FONT_FILE: &str = "fonts/Teko-Medium.ttf";

/// Displays the away team's score.
#[derive(Component)]
pub struct AwayTeamScoreText;

impl AwayTeamScoreText {
    pub fn bundle(asset_server: &Res<AssetServer>) -> impl Bundle {
        (
            AwayTeamScoreText,
            Text(String::from("A")),
            TextColor(Color::WHITE),
            TextFont {
                font: FontSource::Handle(asset_server.load(FONT_FILE)),
                font_size: FontSize::VMin(6.0),
                ..default()
            },
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                align_self: AlignSelf::Center,
                top: Val::Vh(2.0),
                left: Val::Vw(65.0),
                ..default()
            },
        )
    }
}