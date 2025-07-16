use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::*;

use crate::ui::components::*;
use crate::ui::UiSystems;

const FONT_FILE: &str = "fonts/Teko-Medium.ttf";
pub struct UiSetup;

impl Plugin for UiSetup {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            UiSetup::setup
        ).in_set(UiSystems));
    }
}

impl UiSetup {
    pub fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn(
            Node {
                display: Display::Block,
                position_type: PositionType::Absolute,
                top: Val::Px(-4.0),
                left: Val::Px(50.0),
                ..default()
            }
        ).with_children(|parent| {
            parent.spawn((
                Text2d(String::new()),
                TextColor(Color::WHITE),
                TextFont {
                    font: asset_server.load(FONT_FILE),
                    font_size: 36.0,
                    ..default()
                },
                Transform {
                    scale: Vec3::splat(0.07),
                    ..Default::default()
                },
                GameLogPerfText
            ));
        });

        commands.spawn((
            Text2d(String::from("H")),
            TextColor(Color::WHITE),
            TextFont {
                font: asset_server.load(FONT_FILE),
                font_size: 70.0,
                ..default()
            },
            Node {
                display: Display::Block,
                position_type: PositionType::Absolute,
                top: Val::Px(107.0),
                left: Val::Px(30.0),
                ..default()
            },
            Transform {
                scale: Vec3::splat(0.07),
                ..Default::default()
            },
            HomeTeamScoreText
        ));

        commands.spawn((
            Text2d(String::from("")),
            TextColor(Color::WHITE),
            TextFont {
                font: asset_server.load(FONT_FILE),
                font_size: 60.0,
                ..default()
            },
            Node {
                display: Display::Block,
                position_type: PositionType::Absolute,
                top: Val::Px(107.0),
                left: Val::Px(19.0),
                ..default()
            },
            Transform {
                scale: Vec3::splat(0.07),
                ..Default::default()
            },
            HomeTeamScoreUpdateText
        ));

        commands.spawn((
            Text2d(String::from("A")),
            TextColor(Color::WHITE),
            TextFont {
                font: asset_server.load(FONT_FILE),
                font_size: 70.0,
                ..default()
            },
            Node {
                display: Display::Block,
                position_type: PositionType::Absolute,
                top: Val::Px(107.0),
                left: Val::Px(70.0),
                ..default()
            },
            Transform {
                scale: Vec3::splat(0.07),
                ..Default::default()
            },
            AwayTeamScoreText
        ));


        commands.spawn((
            Text2d(String::from("")),
            TextColor(Color::WHITE),
            TextFont {
                font: asset_server.load(FONT_FILE),
                font_size: 60.0,
                ..default()
            },
            Node {
                display: Display::Block,
                position_type: PositionType::Absolute,
                top: Val::Px(107.0),
                left: Val::Px(77.0),
                ..default()
            },
            Transform {
                scale: Vec3::splat(0.07),
                ..Default::default()
            },
            AwayTeamScoreUpdateText
        ));

        commands.spawn((
            Text2d(String::from("0:00")),
            TextFont {
                font: asset_server.load(FONT_FILE),
                font_size: 70.0,
                ..default()
            },
            Node {
                display: Display::Block,
                position_type: PositionType::Absolute,
                top: Val::Px(107.0),
                left: Val::Px(50.0),
                ..default()
            },
            Transform {
                scale: Vec3::splat(0.07),
                ..Default::default()
            },
            MatchTimerText
        ));
    }
}