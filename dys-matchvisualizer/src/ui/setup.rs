use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::prelude::*;
use crate::ui::components::*;

pub struct UiSetup;

impl Plugin for UiSetup {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, UiSetup::setup);
    }
}

impl UiSetup {
    pub fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn((
            // Parent UI node
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                ..default()
            },
        )).with_children(|parent| {
            parent.spawn(GameLogPerfText::bundle(&asset_server));
            let mut comp = parent.spawn(HomeTeamScoreText::bundle(&asset_server));
            comp.log_components();
            parent.spawn(HomeTeamScoreUpdateText::bundle(&asset_server));
            parent.spawn(AwayTeamScoreText::bundle(&asset_server));
            parent.spawn(AwayTeamScoreUpdateText::bundle(&asset_server));
            parent.spawn(MatchTimerText::bundle(&asset_server));
        });
    }
}