use std::sync::{Arc, Mutex};

use dys_simulation::{game_log::GameLog, game_objects::{ball::BallId, combatant::CombatantId}, game_tick::GameTickNumber, simulation::simulation_event::SimulationEvent};

use bevy::{math::vec2, prelude::*, sprite::MeshMaterial2d};
use bevy::prelude::Color::Srgba;
use bevy::render::camera::ScalingMode;
use bevy::sprite::AlphaMode2d;
use bevy::window::WindowResolution;
use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::wasm_bindgen;
use web_time::{Duration, Instant};

#[cfg(target_family = "wasm")]
use bevy::asset::AssetMetaCheck;
use dys_world::combatant::instance::CombatantInstanceId;

const FONT_FILE: &str = "fonts/Teko-Medium.ttf";

fn main() {
    // Set the static memory to None for Option<VisualizationState>,
    // indicating we have no game state to update within the Bevy app.
    UPDATED_VIS_STATE.set(
        Arc::new(Mutex::new(None))
    ).expect("failed to set initial updated visualization state from web process");

    #[cfg(not(target_family="wasm"))]
    {
        restart_with_local_game_log();
        initialize_with_canvas(String::new());
    }
}

#[derive(Component)]
struct GameLogPerfText;

#[derive(Component)]
struct MatchTimerText;

#[derive(Component)]
struct HomeTeamScoreText;

#[derive(Component)]
struct HomeTeamScoreUpdateText;

#[derive(Component)]
struct AwayTeamScoreText;

#[derive(Component)]
struct AwayTeamScoreUpdateText;

#[derive(Component)]
struct PostgameScoreboard;

#[derive(Component)]
struct CombatantIdText {
    combatant_id: CombatantId,
    is_stunned: bool,
}

#[derive(Clone, Debug)]
enum VisualizationMode {
    /// The visualization will not progress
    Paused,

    /// The visualization will progress only when the viewer instructs it to
    Step,

    /// The visualization will progress without user intervention
    Play,
}

#[derive(Debug, Resource)]
struct VisualizationState {
    should_exit: bool,
    game_log: Option<GameLog>,
    world_state: Option<dys_world::world::World>,
    current_tick: GameTickNumber,
    last_update_time: Instant,
    home_score: u16,
    home_score_diff_from_last_tick: i16,
    away_score: u16,
    away_score_diff_from_last_tick: i16,
    end_of_game: bool,
    mode: VisualizationMode,
}

/// This is quite the hack.
///
/// Once we start the Bevy app, we don't have a handle to the running process any more,
/// and can't update the VisualizationState resource in the standard Bevy ways.
///
/// To get around this, we have this static OnceCell, that holds an Arc<Mutex<Option<VisualizationState>>>.
/// It is initialized in [initialize_with_canvas()] to hold a None value for the Option.
static UPDATED_VIS_STATE: OnceCell<Arc<Mutex<Option<VisualizationState>>>> = OnceCell::new();

/// All objects in the simulation visualization will have this component.
/// This allows us to easily clean up if the user wants to reload/leave the visualization.
#[derive(Component)]
struct VisualizationObject;

#[derive(Component)]
struct CombatantVisualizer {
    pub id: CombatantId,
    pub instance_id: CombatantInstanceId,
    pub desired_location: Vec3,
    pub last_position: Vec3,
}

#[derive(Component)]
struct BallVisualizer {
    pub id: BallId,
    pub desired_location: Vec3,
    pub last_position: Vec3,
    pub desired_scale: Vec3,
    pub last_scale: Vec3,
}

#[derive(Component)]
struct ExplosionVisualizer {
    pub opacity: u8,
}

#[derive(Component)]
struct BarrierVisualizer;

#[derive(Component)]
struct PlateVisualizer;

#[wasm_bindgen(js_name = initializeWithCanvas)]
pub fn initialize_with_canvas(
    canvas_id: String
) {
    let canvas: Option<String> = if canvas_id.is_empty() { None } else { Some(canvas_id) };

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        name: Some(String::from("Match Visualizer")),
                        canvas,
                        resolution: WindowResolution::new(1200.0, 1200.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // In WASM releases, we won't have an "assets" dir
                    // Just assume the files exist all within the current path
                    #[cfg(target_family = "wasm")]
                    file_path: String::new(),

                    // Bevy also validates the existence of .meta files,
                    // which we won't have when serving files.
                    #[cfg(target_family = "wasm")]
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        ))
        .insert_resource(VisualizationState {
            should_exit: false,
            game_log: None,
            world_state: None,
            current_tick: 0,
            last_update_time: Instant::now(),
            home_score: 0,
            home_score_diff_from_last_tick: 0,
            away_score: 0,
            away_score_diff_from_last_tick: 0,
            end_of_game: false,
            mode: VisualizationMode::Paused,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update,
            display_game_log_perf,
            display_current_score,
            handle_keyboard_input,
            update_explosion_visualizers,
            update_plate_visualizers,
            update_scoring_text_visualizers.after(update),
            update_combatant_id_text,
            try_reload_vis_state.before(update),
            update_postgame_scoreboard.after(try_reload_vis_state),
        ))
        .run();
}

fn restart_with_local_game_log() {
    #[cfg(not(target_family = "wasm"))]
    {
        let game_log_bytes = include_bytes!("../data/game_log.bin");
        let world_state_bytes = include_bytes!("../data/world_state.bin");
        load_game_log(game_log_bytes.to_vec(), world_state_bytes.to_vec());
    }
}

#[wasm_bindgen(js_name = loadGameLog)]
pub fn load_game_log(
    serialized_game_log: Vec<u8>,
    serialized_world_state: Vec<u8>,
) {
    let game_log: GameLog = postcard::from_bytes(&serialized_game_log).expect("failed to deserialize game log");
    let world: dys_world::world::World = serde_json::from_str(std::str::from_utf8(serialized_world_state.as_slice()).unwrap()).expect("failed to deserialize world state");

    let updated_visualization_state = UPDATED_VIS_STATE.get().unwrap();

    #[cfg(not(target_family="wasm"))]
    let visualization_mode = VisualizationMode::Step;

    #[cfg(target_family="wasm")]
    let visualization_mode = VisualizationMode::Play;

    updated_visualization_state.lock().unwrap().replace(VisualizationState {
        should_exit: false,
        game_log: Some(game_log),
        world_state: Some(world),
        current_tick: 0,
        last_update_time: Instant::now(),
        home_score: 0,
        home_score_diff_from_last_tick: 0,
        away_score: 0,
        away_score_diff_from_last_tick: 0,
        end_of_game: false,
        mode: visualization_mode,
    });
}

#[wasm_bindgen]
pub fn exit() {
    UPDATED_VIS_STATE
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .replace(VisualizationState {
            should_exit: true,
            game_log: None,
            world_state: None,
            current_tick: 0,
            last_update_time: Instant::now(),
            home_score: 0,
            home_score_diff_from_last_tick: 0,
            away_score: 0,
            away_score_diff_from_last_tick: 0,
            end_of_game: false,
            mode: VisualizationMode::Paused,
        });
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection{
            near: -100.0, // Default sets this to zero, when it should be negative
            far: 1000.0,
            scale: 0.13,
            viewport_origin: Default::default(),
            scaling_mode: ScalingMode::Fixed {
                width: 900.0,
                height: 900.0,
            },
            area: Default::default(),
        }),
        Transform::from_xyz(-8.0, -6.0, 0.0),
    ));

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

fn try_reload_vis_state(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut vis_state: ResMut<VisualizationState>,
    asset_server: Res<AssetServer>,
    entity_query: Query<Entity, Or<(With<VisualizationObject>, With<Text>)>>,
) {
    // If we don't have pending updated game state from WASM, abort early
    let Some(updated_vis_state) = UPDATED_VIS_STATE.get() else {
        return;
    };

    let Some(new_vis_state) = updated_vis_state.lock().unwrap().take() else {
        return;
    };

    // We have new game state - blow away all of our current state
    for entity in &entity_query {
        commands.entity(entity).despawn();
    }

    *vis_state = new_vis_state;

    setup_after_reload_game_log(commands, meshes, materials, asset_server, vis_state);
}

fn setup_after_reload_game_log(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    vis_state: ResMut<VisualizationState>
) {
    // If we're in the exit state, do nothing
    if vis_state.should_exit {
        return;
    }

    // This function assumes we're setting up state from tick zero
    // Maybe there's a world where you can live-watch matches and want to join in some intermediate state, but that's not this world
    assert!(vis_state.current_tick == 0);
    let game_log = vis_state.game_log.as_ref().unwrap();
    assert!(!game_log.ticks().is_empty());

    let tick_zero = game_log.ticks().iter().next().unwrap();
    assert!(tick_zero.tick_number == 0);

    for evt in &tick_zero.simulation_events {
        match evt {
            SimulationEvent::ArenaObjectPositionUpdate { object_type_id, position, scale, rotation } => {
                let z_index = match object_type_id {
                    1 => -10.0,
                    2 => -9.0,
                    3 => -8.0,
                    4 => -7.0,
                    _ => -50.0,
                };
                let translation = Vec3::new(position.x, position.z, z_index);
                let transform = Transform {
                    translation,
                    rotation: Quat::from_xyzw(rotation.i, rotation.k, rotation.j, rotation.w),
                    scale: Vec3::new(1.0, 1.0, 1.0), // ZJ-TODO
                };

                let color = match object_type_id {
                    1 => Color::linear_rgb(0.2, 0.2, 0.2),
                    2 => Color::linear_rgb(0.2, 0.2, 0.8),
                    3 => Color::linear_rgb(0.5, 0.5, 0.5),
                    4 => Color::linear_rgb(0.7, 0.0, 0.7),
                    _ => Color::linear_rgb(0.0, 0.0, 0.0),
                };

                let mesh_shape: Mesh = match object_type_id {
                    4 => Circle { radius: scale.x }.into(),
                    _ => Rectangle { half_size: vec2(scale.x / 2.0, scale.z / 2.0) }.into(),
                };

                let mut entity_commands = commands.spawn((
                    VisualizationObject,
                    Mesh2d(meshes.add(mesh_shape)),
                    MeshMaterial2d(materials.add(color)),
                    transform,
                ));

                if *object_type_id == 4 {
                    entity_commands.insert(PlateVisualizer);

                    let mut inner_transform = transform;
                    // z-ordering...
                    inner_transform.translation.z += 1.0;
                    commands.spawn((
                       VisualizationObject,
                       Mesh2d(meshes.add(Circle { radius: scale.x * 0.9 })),
                       MeshMaterial2d(materials.add(color)),
                       inner_transform,
                    ));
                } else {
                    entity_commands.insert(BarrierVisualizer);
                }
            },
            SimulationEvent::BallPositionUpdate { ball_id, position } => {
                // ZJ-TODO: adding 10.0 sucks, but is necessary for balls to show above combatants in z-ordering
                //          otherwise the ball can "hide" under combatants
                let translation = Vec3::new(position.x, position.z, position.y + 10.0);
                let transform = Transform {
                    translation,
                    rotation: Quat::default(),
                    scale: Vec3::ONE,
                };
                commands.spawn((
                    VisualizationObject,
                    BallVisualizer {
                        id: *ball_id,
                        desired_location: translation,
                        last_position: translation,
                        desired_scale: Vec3::ONE,
                        last_scale: Vec3::ONE
                    },
                    Mesh2d(meshes.add(Circle { radius: 0.5 })), // ZJ-TODO: read radius from ball object
                    MeshMaterial2d(materials.add(Color::linear_rgb(0.75, 0.75, 0.0))),
                    transform,
                ));
            },
            SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                let translation = Vec3::new(position.x, position.z, position.y);
                let transform = Transform {
                    translation,
                    rotation: Quat::default(),
                    scale: Vec3::ONE, // ZJ-TODO
                };

                // ZJ-TODO: don't assume team by position
                let home_team = position.x < 50.0;

                let instance_id = game_log
                    .combatant_id_mapping()
                    .get(combatant_id)
                    .unwrap()
                    .to_owned();

                let world = vis_state.world_state.as_ref().unwrap();
                let mut name = String::new();
                for combatant in &world.combatants {
                    let combatant_instance = combatant.lock().unwrap();
                    if combatant_instance.id == instance_id {
                        name = combatant_instance.name.to_owned();
                        break;
                    }
                }

                if name.is_empty() {
                    panic!("failed to find combatant with instance ID {instance_id}");
                };

                name = name.splitn(2, " ").skip(1).collect::<String>();

                commands.spawn((
                    VisualizationObject,
                    CombatantVisualizer {
                        id: *combatant_id,
                        instance_id,
                        desired_location: translation,
                        last_position: translation
                    },
                    Mesh2d(meshes.add(Capsule2d::new(0.75, 1.75))), // ZJ-TODO: read radius
                    MeshMaterial2d(materials.add(Color::linear_rgb(
                        0.0,
                        if home_team { 0.4 } else { 0.0 },
                        if home_team { 0.0 } else { 1.0 },
                    ))),
                    transform,
                )).with_children(|builder| {
                    // We'll inherit the x and y coords from the parent,
                    // but we want this text to always be on top, and to be smooth
                    let mut new_transform = Transform::default();
                    new_transform.translation.z = 30.0;
                    new_transform.translation.y = -4.0;
                    new_transform.scale *= 0.05;

                    builder.spawn((
                        Text2d(name),
                        TextFont {
                            font: asset_server.load(FONT_FILE),
                            font_size: 64.0,
                            ..Default::default()
                        },
                        TextColor(Color::WHITE),
                        new_transform,
                        VisualizationObject,
                        CombatantIdText {
                            combatant_id: *combatant_id,
                            is_stunned: false,
                        },
                    ));
                });
            },
            _ => {}, // ZJ-TODO: we should assert if we have any unexpected events
        }
    }

    let combatant_statlines = game_log.combatant_statlines().to_owned();
    commands.spawn((
        Node {
            top: Val::Percent(10.0),
            left: Val::Percent(10.0),
            height: Val::Percent(80.0),
            width: Val::Percent(80.0),
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        VisualizationObject,
        PostgameScoreboard,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                min_width: Val::Percent(100.0),
                height: Val::Percent(9.0),
                min_height: Val::Percent(9.0),
                ..default()
            },
        )).with_children(|parent| {
            for stat_category in [
                "Combatant",
                "Points",
                "Throws",
                "Hits",
                "Shoves",
            ] {
                parent.spawn(
                    Node {
                        width: Val::Percent(100.0),
                        ..default()
                    }
                ).with_child((
                    Text::new(stat_category),
                    TextFont {
                        font: asset_server.load(FONT_FILE),
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            }
        });

        let combatant_mapping = vis_state.game_log.as_ref().unwrap().combatant_id_mapping();
        let world = vis_state.world_state.as_ref().unwrap();
        for statline in combatant_statlines {
            let combatant_instance_id = combatant_mapping.get(&statline.combatant_id).unwrap();
            let mut combatant_name = statline.combatant_id.to_string();
            for combatant in &world.combatants {
                let combatant_instance = combatant.lock().unwrap();
                if combatant_instance.id == *combatant_instance_id {
                    combatant_name = combatant_instance.name.splitn(2, " ").skip(1).collect::<String>();
                    break;
                }
            }

            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    min_width: Val::Percent(100.0),
                    height: Val::Percent(9.0),
                    min_height: Val::Percent(9.0),
                    ..default()
                },
            )).with_children(|parent| {
                for stat_str in [
                    combatant_name,
                    statline.points_scored.to_string(),
                    statline.balls_thrown.to_string(),
                    statline.throws_hit.to_string(),
                    statline.combatants_shoved.to_string(),
                ] {
                    parent.spawn(
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        }
                    ).with_child((
                        Text::new(stat_str),
                        TextFont {
                            font: asset_server.load(FONT_FILE),
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });
        }
    });
}

fn update(
    mut commands: Commands,
    mut vis_state: ResMut<VisualizationState>,
    mut combatants_query: Query<(&mut CombatantVisualizer, &mut Transform), Without<BallVisualizer>>,
    mut combatant_id_text_query: Query<&mut CombatantIdText>,
    mut balls_query: Query<(&mut BallVisualizer, &mut Transform), Without<CombatantVisualizer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if matches!(vis_state.mode, VisualizationMode::Paused) {
        return;
    }

    if vis_state.should_exit {
        return;
    }

    // Only update the simulation every second, otherwise would be too fast
    // ZJ-TODO: allow this to be configurable
    const TICKS_PER_SECOND: u64 = 10;
    const TIME_BETWEEN_TICKS_MILLIS: u64 = 1000 / TICKS_PER_SECOND;
    const TIME_BETWEEN_TICKS: Duration = Duration::from_millis(TIME_BETWEEN_TICKS_MILLIS);
    let time_since_last_update = Instant::now() - vis_state.last_update_time;
    let lerp_progress = ((time_since_last_update.as_millis() as u64 % 1000) as f32 / 100.0)
        .clamp(0.0, 1.0);

    // Visual updates can occur ever frame
    for (mut combatant_vis, mut combatant_transform) in combatants_query.iter_mut() {
        combatant_transform.translation = combatant_vis.last_position.lerp(
            combatant_vis.desired_location,
            lerp_progress
        );

        combatant_vis.last_position = combatant_transform.translation;
        debug!("Workaround for ignored field: {}", combatant_vis.instance_id);
    }

    for (mut ball_vis, mut ball_transform) in balls_query.iter_mut() {
        ball_transform.translation = ball_vis.last_position.lerp(
            ball_vis.desired_location,
            lerp_progress
        );

        ball_vis.last_position = ball_transform.translation;

        ball_transform.scale = ball_vis.last_scale.lerp(
            ball_vis.desired_scale,
            lerp_progress
        );

        ball_vis.last_scale = ball_transform.scale;
    }

    if time_since_last_update < TIME_BETWEEN_TICKS {
        return;
    }

    if matches!(vis_state.mode, VisualizationMode::Play) && !vis_state.end_of_game {
        vis_state.current_tick += 1;
    }

    let mut new_home_score = vis_state.home_score;
    let mut new_away_score = vis_state.away_score;

    let current_tick = vis_state.current_tick;
    {
        let events_this_tick = {
            let game_log = vis_state.game_log.as_ref().unwrap();
            game_log.ticks().get(current_tick as usize)
        };

        if events_this_tick.is_none() {
            vis_state.end_of_game = true;
            return;
        }

        let events_this_tick = events_this_tick.unwrap();

        for event in &events_this_tick.simulation_events {
            match event {
                SimulationEvent::ArenaObjectPositionUpdate { .. } => { /* no-op, nothing to move with arena objects currently - this may change if plates start moving */},
                SimulationEvent::BallPositionUpdate { ball_id, position } => {
                    let (mut ball_vis, _) = balls_query.iter_mut()
                        .find(|(ball_vis, _)| ball_vis.id == *ball_id)
                        .unwrap();

                    ball_vis.desired_location = Vec3::new(position.x, position.z, position.y);
                    // Every 3 units vertically, make the ball twice as big
                    let scale_modifier = (1.0 + (position.y / 3.0)).max(1.0);
                    ball_vis.desired_scale = Vec3::ONE * scale_modifier;
                },
                SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                    let (mut combatant_vis, _) = combatants_query.iter_mut()
                        .find(|(combatant_vis, _)| combatant_vis.id == *combatant_id)
                        .unwrap();

                    combatant_vis.desired_location = Vec3::new(position.x, position.z, position.y);
                },
                SimulationEvent::PointsScoredByCombatant { plate_id: _, combatant_id, points } => {
                    let is_home_team = *combatant_id <= 5;
                    if is_home_team {
                        new_home_score += *points as u16;
                    } else {
                        new_away_score += *points as u16;
                    }
                },
                SimulationEvent::BallExplosion { ball_id, charge } => {
                    let (_, ball_pos) = balls_query.iter()
                        .find(|(ball_vis, _)| ball_vis.id == *ball_id)
                        .unwrap();

                    let explosion_radius = charge * 0.3;
                    let mut color_material = ColorMaterial::from_color(Color::srgba(1.0, 0.4, 0.0, 1.0));
                    color_material.alpha_mode = AlphaMode2d::Blend;
                    commands.spawn((
                        VisualizationObject,
                        ExplosionVisualizer { opacity: 100 },
                        Mesh2d(meshes.add(Circle::new(explosion_radius))),
                        MeshMaterial2d(materials.add(color_material)),
                        ball_pos.to_owned(),
                    ));
                },
                SimulationEvent::CombatantStunned { combatant_id, start } => {
                    for mut combatant_id_text in combatant_id_text_query.iter_mut() {
                        if combatant_id_text.combatant_id != *combatant_id {
                            continue;
                        }

                        combatant_id_text.is_stunned = *start;
                        break;
                    }
                },
                _ => {}
            }
        }
    }

    vis_state.home_score_diff_from_last_tick = new_home_score as i16 - vis_state.home_score as i16;
    vis_state.away_score_diff_from_last_tick = new_away_score as i16 - vis_state.away_score as i16;
    vis_state.home_score = new_home_score;
    vis_state.away_score = new_away_score;
    vis_state.last_update_time = Instant::now();
}

fn display_game_log_perf(
    vis_state: Res<VisualizationState>,
    mut text_query: Query<&mut Text2d, With<GameLogPerfText>>,
) {
    let Some(ref game_log) = vis_state.game_log else {
        return;
    };

    for mut text in text_query.iter_mut() {
        // Bevy default font doesn't display unicode (or at least 'μ')
        // Just replace with 'u'
        text.0 = game_log.perf_string().replace("μ", "u") + format!(" (tick {})", vis_state.current_tick).as_str();
    }
}

fn display_current_score(
    mut set: ParamSet<(
        Query<(&mut Text2d, &mut TextColor), With<HomeTeamScoreText>>,
        Query<(&mut Text2d, &mut TextColor), With<AwayTeamScoreText>>,
        Query<&mut Text2d, With<MatchTimerText>>,
    )>,
    vis_state: Res<VisualizationState>,
) {
    if vis_state.should_exit {
        return;
    }

    const TICKS_PER_SECOND: u32 = 10;
    {
        let mut home_text_query = set.p0();
        let (mut home_text, mut color) = home_text_query.single_mut().expect("failed to get home score text component");
        home_text.0 = format!("{}", vis_state.home_score);
        if vis_state.end_of_game {
            if vis_state.home_score >= vis_state.away_score {
                color.0 = Color::srgb(0.0, 1.0, 0.0);
            } else {
                color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
        } else {
            color.0 = Color::WHITE;
        }
    }

    {
        let mut away_text_query = set.p1();
        let (mut away_text, mut color) = away_text_query.single_mut().expect("failed to get away score text component");
        away_text.0 = format!("{}", vis_state.away_score);
        if vis_state.end_of_game {
            if vis_state.away_score >= vis_state.home_score {
                color.0 = Color::srgb(0.0, 1.0, 0.0);
            } else {
                color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
        } else {
            color.0 = Color::WHITE;
        }
    }

    {
        let mut match_timer_text_query = set.p2();
        let mut match_timer_text = match_timer_text_query.single_mut().expect("failed to get match timer text component");
        let minutes_component = vis_state.current_tick / TICKS_PER_SECOND / 60;
        let seconds_component = vis_state.current_tick / TICKS_PER_SECOND % 60;
        match_timer_text.0 = format!("{minutes_component}:{seconds_component:02}");
    }
}

fn update_explosion_visualizers(
    mut commands: Commands,
    mut explosion_query: Query<(&mut ExplosionVisualizer, &mut MeshMaterial2d<ColorMaterial>, Entity)>,
    mut assets: ResMut<Assets<ColorMaterial>>
) {
    let decrement_amount = 2.5;

    for (mut explosion, mesh_handle, entity) in explosion_query.iter_mut() {
        let Some(new_opacity) = explosion.opacity.checked_sub(decrement_amount as u8) else {
            commands.entity(entity).despawn();
            continue;
        };

        explosion.opacity = new_opacity;

        let Some(color_material) = assets.get_mut(mesh_handle.0.id()) else {
            continue;
        };

        let Srgba(mut rgba) = color_material.color else {
            continue;
        };

        rgba.alpha -= decrement_amount / 100.0;
        color_material.color = rgba.into();
    }
}

fn update_plate_visualizers(
    mut plate_query: Query<(&mut PlateVisualizer, &mut MeshMaterial2d<ColorMaterial>)>,
    vis_state: Res<VisualizationState>,
    mut assets: ResMut<Assets<ColorMaterial>>
) {
    for (_, mesh_handle) in plate_query.iter_mut() {
        let Some(color_material) = assets.get_mut(mesh_handle.0.id()) else {
            continue;
        };

        let distance_from_mid = (vis_state.current_tick as i16 % 10 - 5).abs();
        let lerp_progress = ((5 - distance_from_mid) * 2) as f32 / 10.0;

        let color_value = Vec3::new(1.0, 1.0, 1.0)
            .lerp(Vec3::new(1.0, 0.0, 1.0), lerp_progress);

        color_material.color = Color::srgb(color_value[0], color_value[1], color_value[2]);
    }
}

fn update_scoring_text_visualizers(
    vis_state: Res<VisualizationState>,
    mut team_query: Query<
        (&mut Text2d, &mut TextColor, &mut Node, Has<HomeTeamScoreUpdateText>),
        Or<(With<HomeTeamScoreUpdateText>, With<AwayTeamScoreUpdateText>)>
    >
) {
    for (mut text, mut text_color, mut node, is_home_team) in team_query.iter_mut() {
        if vis_state.current_tick % 10 == 0 {
            let (self_score_change, opponent_score_change) = {
                if is_home_team {
                    (vis_state.home_score_diff_from_last_tick, vis_state.away_score_diff_from_last_tick)
                } else {
                    (vis_state.away_score_diff_from_last_tick, vis_state.home_score_diff_from_last_tick)
                }
            };

            if self_score_change > 0 {
                text.0 = format!("+{self_score_change}");

                if opponent_score_change == 0 {
                    *text_color = TextColor(Color::srgb(0.71, 0.58, 0.06));
                } else {
                    *text_color = TextColor(Color::srgb(0.16, 0.45, 0.31));
                }
            } else {
                text.0 = String::new();
                *text_color = TextColor(Color::WHITE);
            }

            node.top = Val::Px(107.0);
        } else {
            let alpha = 1.0 - (vis_state.current_tick % 10) as f32 / 10.0;
            let mut existing_color = text_color.0;
            existing_color.set_alpha(alpha);
            *text_color = TextColor(existing_color);

            node.top = Val::Px(107.0 + ((vis_state.current_tick % 10) as f32 / 2.0));
        }
    }
}

fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut vis_state: ResMut<VisualizationState>,
) {
    if vis_state.should_exit {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Tab) && !matches!(vis_state.mode, VisualizationMode::Play) {
        vis_state.mode = VisualizationMode::Step;
        vis_state.current_tick += 1;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        vis_state.mode = match &vis_state.mode {
            VisualizationMode::Play => VisualizationMode::Paused,
            _ => VisualizationMode::Play,
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        restart_with_local_game_log();
    }
}

fn update_combatant_id_text(
    mut combatants_query: Query<(&mut TextColor, &CombatantIdText)>,
) {
    for (mut text_color, combatant_id_text) in combatants_query.iter_mut() {
        if combatant_id_text.is_stunned {
            *text_color = TextColor(Color::srgb(1.0, 0.0, 0.0));
        } else {
            *text_color = TextColor(Color::WHITE);
        }
    }
}

fn update_postgame_scoreboard(
    mut query: Query<&mut Visibility, With<PostgameScoreboard>>,
    vis_state: Res<VisualizationState>,
) {
    if vis_state.should_exit {
        return;
    }

    let Ok(mut visibility) = query.single_mut() else {
        return;
    };

    if vis_state.end_of_game {
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}