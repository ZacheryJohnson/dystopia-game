use std::sync::{Arc, Mutex};

use dys_simulation::{game_log::GameLog, game_objects::{ball::BallId, combatant::CombatantId}, game_tick::GameTickNumber, simulation::simulation_event::SimulationEvent};

use bevy::{math::{bounding::{Aabb2d, IntersectsVolume}, vec2}, prelude::*, sprite::{MeshMaterial2d}, window::PrimaryWindow};
use bevy::prelude::Color::Srgba;
use bevy::render::camera::ScalingMode;
use bevy::sprite::AlphaMode2d;
use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::wasm_bindgen;
use web_time::{Duration, Instant};

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
struct DebugPositionText;

#[derive(Component)]
struct MatchTimerText;

#[derive(Component)]
struct HomeTeamScoreText;

#[derive(Component)]
struct AwayTeamScoreText;

#[derive(Component)]
struct PostgameScoreboard;

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
    game_log: Option<GameLog>,
    current_tick: GameTickNumber,
    last_update_time: Instant,
    home_score: u16,
    away_score: u16,
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
    pub desired_location: Vec3,
    pub last_position: Vec3,
}

#[derive(Component)]
struct BallVisualizer {
    pub id: BallId,
    pub desired_location: Vec3,
    pub last_position: Vec3,
}

#[derive(Component)]
struct ExplosionVisualizer {
    pub opacity: u8,
}

#[derive(Component)]
struct BarrierVisualizer;

#[wasm_bindgen(js_name = initializeWithCanvas)]
pub fn initialize_with_canvas(
    canvas_id: String
) {
    info!("Initializing with canvas {}", canvas_id);

    let canvas: Option<String> = if canvas_id.is_empty() { None } else { Some(canvas_id) };

    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    name: Some(String::from("Match Visualizer")),
                    canvas,
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
        )
        .insert_resource(VisualizationState {
            game_log: None,
            current_tick: 0,
            last_update_time: Instant::now(),
            home_score: 0,
            away_score: 0,
            end_of_game: false,
            mode: VisualizationMode::Paused,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update,
            display_game_log_perf,
            display_mouse_hover,
            display_current_score,
            handle_keyboard_input,
            update_explosion_visualizers,
            update_postgame_scoreboard,
            try_reload_vis_state.before(update),
        ))
        .run();
}

fn restart_with_local_game_log() {
    let game_log_bytes = include_bytes!("../data/game_log.bin");
    load_game_log(game_log_bytes.to_vec());
}

#[wasm_bindgen(js_name = loadGameLog)]
pub fn load_game_log(
    serialized_game_log: Vec<u8>,
) {
    let game_log: GameLog = postcard::from_bytes(&serialized_game_log).expect("failed to deserialize game log");

    let updated_visualization_state = UPDATED_VIS_STATE.get().unwrap();

    #[cfg(not(target_family="wasm"))]
    let visualization_mode = VisualizationMode::Step;

    #[cfg(target_family="wasm")]
    let visualization_mode = VisualizationMode::Play;

    updated_visualization_state.lock().unwrap().replace(VisualizationState {
        game_log: Some(game_log),
        current_tick: 0,
        last_update_time: Instant::now(),
        home_score: 0,
        away_score: 0,
        end_of_game: false,
        mode: visualization_mode,
    });
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            near: -100.0, // Default sets this to zero, when it should be negative
            far: 1000.0,
            scale: 0.13,
            viewport_origin: Default::default(),
            scaling_mode: ScalingMode::Fixed {
                width: 1600.0,
                height: 900.0,
            },
            area: Default::default(),
        },
        Transform::from_xyz(-50.0, -6.0, 0.0),
    ));

    commands.spawn((
        Text2d(String::new()),
        TextFont {
            font_size: 50.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Vw(2.0),
            left: Val::Vw(50.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Transform {
            scale: Vec3::splat(0.07),
            ..Default::default()
        },
        DebugPositionText
    ));

    commands.spawn((
        Text2d(String::new()),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        Node {
            position_type: PositionType::Relative,
            bottom: Val::Px(4.0),
            left: Val::Px(50.0),
            ..default()
        },
        Transform {
            scale: Vec3::splat(0.07),
            ..Default::default()
        },
        GameLogPerfText
    ));

    commands.spawn((
        Text2d(String::from("H")),
        TextColor(Color::WHITE),
        TextFont {
            font_size: 60.0,
            ..default()
        },
        Node {
            position_type: PositionType::Relative,
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
        Text2d(String::from("A")),
        TextColor(Color::WHITE),
        TextFont {
            font_size: 60.0,
            ..default()
        },
        Node {
            position_type: PositionType::Relative,
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
        Text2d(String::from("0:00")),
        TextFont {
            font_size: 60.0,
            ..default()
        },
        Node {
            position_type: PositionType::Relative,
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

    commands.spawn((
        Text2d(String::from("")),
        TextFont {
            font_size: 120.0,
            ..default()
        },
        Node {
            position_type: PositionType::Relative,
            top: Val::Px(50.0),
            left: Val::Px(50.0),
            ..default()
        },
        Transform {
            scale: Vec3::splat(0.07),
            ..Default::default()
        },
        PostgameScoreboard
    ));
}

fn try_reload_vis_state(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut vis_state: ResMut<VisualizationState>,
    entity_query: Query<Entity, With<VisualizationObject>>,
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

    setup_after_reload_game_log(commands, meshes, materials, vis_state);
}

fn setup_after_reload_game_log(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    vis_state: ResMut<VisualizationState>
) {
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
                    translation: translation.clone(),
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

                commands.spawn((
                    VisualizationObject,
                    BarrierVisualizer, // ZJ-TODO: assuming barrier here, probably not the right idea
                    Mesh2d(meshes.add(mesh_shape)),
                    MeshMaterial2d(materials.add(color)),
                    transform,
                ));
            },
            SimulationEvent::BallPositionUpdate { ball_id, position } => {
                // ZJ-TODO: adding 10.0 sucks, but is necessary for balls to show above combatants in z-ordering
                //          otherwise the ball can "hide" under combatants
                let translation = Vec3::new(position.x, position.z, position.y + 10.0);
                let transform = Transform {
                    translation: translation.clone(),
                    rotation: Quat::default(),
                    scale: Vec3::ONE, // ZJ-TODO
                };
                commands.spawn((
                    VisualizationObject,
                    BallVisualizer { id: *ball_id, desired_location: translation, last_position: translation },
                    Mesh2d(meshes.add(Circle { radius: 1.0 })), // ZJ-TODO: read radius from ball object
                    MeshMaterial2d(materials.add(Color::linear_rgb(0.75, 0.75, 0.0))),
                    transform,
                ));
            },
            SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                let translation = Vec3::new(position.x, position.z, position.y);
                let transform = Transform {
                    translation: translation.clone(),
                    rotation: Quat::default(),
                    scale: Vec3::ONE, // ZJ-TODO
                };

                // ZJ-TODO: don't assume team by position
                let home_team = if position.x < 50.0 { true } else { false };

                commands.spawn((
                    VisualizationObject,
                    CombatantVisualizer { id: *combatant_id, desired_location: translation, last_position: translation },
                    Mesh2d(meshes.add(Capsule2d::new(1.0, 2.0))), // ZJ-TODO: read radius
                    MeshMaterial2d(materials.add(Color::linear_rgb(
                        0.0,
                        if home_team { 1.0 } else { 0.0 },
                        if home_team { 0.0 } else { 1.0 },
                    ))),
                    transform.clone(),
                )).with_children(|builder| {
                    // We'll inherit the x and y coords from the parent,
                    // but we want this text to always be on top, and to be smooth
                    let mut new_transform = Transform::default();
                    new_transform.translation.z = 30.0;
                    new_transform.scale *= 0.07;

                    builder.spawn((
                        Text2d(combatant_id.to_string()),
                        TextFont {
                                font_size: 64.0,
                                ..Default::default()
                        },
                        TextColor(if home_team { Color::BLACK } else { Color::WHITE }),
                        new_transform,
                        VisualizationObject
                    ));
                });
            },
            _ => {}, // ZJ-TODO: we should assert if we have any unexpected events
        }
    }
}

fn update(
    mut commands: Commands,
    mut vis_state: ResMut<VisualizationState>,
    mut combatants_query: Query<(&mut CombatantVisualizer, &mut Transform), Without<BallVisualizer>>,
    mut balls_query: Query<(&mut BallVisualizer, &mut Transform), Without<CombatantVisualizer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if matches!(vis_state.mode, VisualizationMode::Paused) {
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
            lerp_progress);

        combatant_vis.last_position = combatant_transform.translation;
    }

    for (mut ball_vis, mut ball_transform) in balls_query.iter_mut() {
        ball_transform.translation = ball_vis.last_position.lerp(
            ball_vis.desired_location,
            lerp_progress);

        ball_vis.last_position = ball_transform.translation;
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
            game_log.ticks().iter().nth(current_tick as usize)
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
                        .filter(|(ball_vis, _)| ball_vis.id == *ball_id)
                        .next()
                        .unwrap();

                    ball_vis.desired_location = Vec3::new(position.x, position.z, position.y);
                },
                SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                    let (mut combatant_vis, _) = combatants_query.iter_mut()
                        .filter(|(combatant_vis, _)| combatant_vis.id == *combatant_id)
                        .next()
                        .unwrap();

                    combatant_vis.desired_location = Vec3::new(position.x, position.z, position.y);
                },
                SimulationEvent::PointsScoredByCombatant { plate_id: _, combatant_id, points } => {
                    let is_home_team = *combatant_id <= 5;
                    if is_home_team {
                        new_home_score = new_home_score + (*points as u16);
                    } else {
                        new_away_score = new_away_score + (*points as u16);
                    }
                },
                SimulationEvent::BallExplosion { ball_id, charge } => {
                    let (_, ball_pos) = balls_query.iter()
                        .filter(|(ball_vis, _)| ball_vis.id == *ball_id)
                        .next()
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
                _ => {}
            }
        }
    }

    vis_state.home_score = new_home_score;
    vis_state.away_score = new_away_score;
    vis_state.last_update_time = Instant::now();
}

fn display_game_log_perf(
    vis_state: Res<VisualizationState>,
    mut text_query: Query<&mut Text2d, With<GameLogPerfText>>,
) {
    let mut text = text_query.get_single_mut().expect("failed to get debug position text component");
    let Some(ref game_log) = vis_state.game_log else {
        return;
    };

    // Bevy default font doesn't display unicode (or at least 'μ')
    // Just replace with 'u'
    text.0 = game_log.perf_string().replace("μ", "u") + format!(" (tick {})", vis_state.current_tick).as_str();
}

fn display_mouse_hover(
    mut camera_query: Query<(&Camera, &GlobalTransform)>,
    vis_objects_query: Query<&Transform, Or<(With<CombatantVisualizer>, With<BallVisualizer>)>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut text_query: Query<&mut Text2d, With<DebugPositionText>>,
) {
    let mut text = text_query.get_single_mut().expect("failed to get debug position text component");
    text.0 = String::new();

    let Ok((camera, camera_global_transform)) = camera_query.get_single_mut() else {
        return;
    };

    let window = window_query
        .get_single()
        .expect("failed to get primary camera");

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // use it to convert ndc to world-space coordinates
    let Ok(world_pos) =
        camera.viewport_to_world_2d(camera_global_transform, cursor_position)
    else {
        // Couldn't convert - mouse likely outside of window
        // Don't log - this would get spammy
        return;
    };

    let world_pos_collider = Aabb2d::new(world_pos, Vec2 { x: 1.0, y: 1.0 });
    for vis_object_transform in &vis_objects_query {
        let vis_object_collider = Aabb2d::new(
            Vec2::new(vis_object_transform.translation.x, vis_object_transform.translation.y),
            Vec2 { x: vis_object_transform.scale.x, y: vis_object_transform.scale.y });
        if !world_pos_collider.intersects(&vis_object_collider) {
            continue;
        }

        text.0 = format!(
            "({}, {}, {})",
            vis_object_transform.translation.x.round(),
            vis_object_transform.translation.y.round(),
            vis_object_transform.translation.z.round(),
        );
        return;
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
    const TICKS_PER_SECOND: u32 = 10;
    {
        let mut home_text_query = set.p0();
        let (mut home_text, mut color) = home_text_query.get_single_mut().expect("failed to get home score text component");
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
        let (mut away_text, mut color) = away_text_query.get_single_mut().expect("failed to get away score text component");
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
        let mut match_timer_text = match_timer_text_query.get_single_mut().expect("failed to get match timer text component");
        let minutes_component = vis_state.current_tick / TICKS_PER_SECOND / 60;
        let seconds_component = vis_state.current_tick / TICKS_PER_SECOND % 60;
        match_timer_text.0 = format!(
            "{}:{:02}",
            minutes_component,
            seconds_component,
        );
    }
}

fn update_explosion_visualizers(
    mut commands: Commands,
    mut explosion_query: Query<(&mut ExplosionVisualizer, &mut MeshMaterial2d<ColorMaterial>, Entity)>,
    mut assets: ResMut<Assets<ColorMaterial>>
) {
    let decrement_amount = 1.0;

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

        rgba.alpha -= 0.01;
        color_material.color = rgba.into();
    }
}

fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut vis_state: ResMut<VisualizationState>,
) {
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

fn update_postgame_scoreboard(
    mut scoreboard_query: Query<&mut Text2d, With<PostgameScoreboard>>,
    vis_state: ResMut<VisualizationState>,
) {
    if vis_state.end_of_game {
        let mut scoreboard_text = scoreboard_query.single_mut();
        scoreboard_text.0 = String::from("GAME OVER!");
    } else {
        let mut scoreboard_text = scoreboard_query.single_mut();
        scoreboard_text.0 = String::new();
    }
}