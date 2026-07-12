use bevy::prelude::*;

const FONT_FILE: &str = "fonts/Teko-Medium.ttf";

/// Shows simulation performance statistics in the visualization.
/// Intended to be dev-only to understand simulation perf at a glance (particularly in web/WASM).
#[derive(Component)]
pub struct GameLogPerfText;

impl GameLogPerfText {
    pub fn bundle(asset_server: &Res<AssetServer>) -> impl Bundle {
        (
            GameLogPerfText,
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                top: Val::VMax(96.0),
                ..default()
            },
            Text(String::new()),
            TextColor(Color::WHITE),
            TextFont {
                font: FontSource::Handle(asset_server.load(FONT_FILE)),
                font_size: FontSize::VMin(3.0),
                ..default()
            },
        )
    }
}