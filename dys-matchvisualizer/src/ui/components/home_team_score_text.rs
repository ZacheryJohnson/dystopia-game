use bevy::prelude::*;

const FONT_FILE: &str = "fonts/Teko-Medium.ttf";

/// Displays the home team's score.
#[derive(Component)]
pub struct HomeTeamScoreText;

impl HomeTeamScoreText {
    pub fn bundle(asset_server: &Res<AssetServer>) -> impl Bundle {
        (
            HomeTeamScoreText,
            Text(String::from("H")),
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
                left: Val::Vw(30.0),
                ..default()
            }
        )
    }
}