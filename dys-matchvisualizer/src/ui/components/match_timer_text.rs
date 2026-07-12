use bevy::prelude::*;

const FONT_FILE: &str = "fonts/Teko-Medium.ttf";

/// Displays the current game time.
#[derive(Component)]
pub struct MatchTimerText;

impl MatchTimerText {
    pub fn bundle(asset_server: &Res<AssetServer>) -> impl Bundle {
        (
            MatchTimerText,
            Text(String::from("0:00")),
            TextFont {
                font: FontSource::Handle(asset_server.load(FONT_FILE)),
                font_size: FontSize::VMin(8.0),
                ..default()
            },
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                top: Val::Vh(0.5),
                ..default()
            },
        )
    }
}