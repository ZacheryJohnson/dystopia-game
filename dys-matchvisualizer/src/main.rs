use std::sync::{Arc, Mutex};

use dys_simulation::{game_log::GameLog, game_objects::{ball::BallId, combatant::CombatantId}, game_tick::GameTickNumber, simulation::simulation_event::SimulationEvent};

use bevy::{math::{bounding::{Aabb2d, IntersectsVolume}, vec2}, prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}, window::PrimaryWindow};
use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::wasm_bindgen;
use web_time::{Duration, Instant};

fn main() {
    // Set the static memory to None for Option<GameState>,
    // indicating we have no game state to update within the Bevy app.
    UPDATED_GAME_STATE.set(
        Arc::new(Mutex::new(None))
    ).expect("failed to set initial updated game state from web process");

    #[cfg(not(target_family="wasm"))]
    {
        let game_log_bytes = include_bytes!("../data/game_log.bin");
        load_game_log(game_log_bytes.to_vec());
        initialize_with_canvas(String::new());
    }
}

#[derive(Component)]
struct GameLogPerfText;

#[derive(Component)]
struct DebugPositionText;

#[derive(Debug)]
enum VisualizationMode {
    /// THe visualization will not progress
    Paused,

    /// The visualization will progress only when the viewer instructs it to
    Step,

    /// The visualization will progress without user intervention
    Play,
}

#[derive(Debug, Resource)]
struct GameState {
    game_log: Option<GameLog>,
    current_tick: GameTickNumber,
    last_update_time: Instant,
    mode: VisualizationMode,
}

/// This is quite the hack.
/// 
/// Once we start the Bevy app, we don't have a handle to the running process any more,
/// and can't update the GameState resource in the standard Bevy ways.
/// 
/// To get around this, we have this static OnceCell, that holds an Arc<Mutex<Option<GameState>>>.
/// It is initialized in [initialize_with_canvas()] to hold a None value for the Option.
static UPDATED_GAME_STATE: OnceCell<Arc<Mutex<Option<GameState>>>> = OnceCell::new();

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
        .insert_resource(GameState {
            game_log: None,
            current_tick: 0,
            last_update_time: Instant::now(),
            mode: VisualizationMode::Paused,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update,
            display_game_log_perf,
            display_mouse_hover,
            try_reload_game_state.before(update),
        ))
        .run();
}

#[wasm_bindgen(js_name = loadGameLog)]
pub fn load_game_log(
    serialized_game_log: Vec<u8>,
) {
    let game_log: GameLog = postcard::from_bytes(&serialized_game_log).expect("failed to deserialize game log");

    let updated_game_state = UPDATED_GAME_STATE.get().unwrap();

    updated_game_state.lock().unwrap().replace(GameState {
        game_log: Some(game_log),
        current_tick: 0,
        last_update_time: Instant::now(),
        mode: VisualizationMode::Play,
    });
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            near: -100.0, // Default sets this to zero, when it should be negative
            far: 1000.0,
            scale: 0.1667,
            ..default()
        },
        transform: Transform::from_xyz(50.0, 50.0, 0.0),
        ..default()
    });

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font_size: 50.0,
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
        DebugPositionText
    ));

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font_size: 30.0,
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        GameLogPerfText
    ));
}

fn try_reload_game_state(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut game_state: ResMut<GameState>,
    entity_query: Query<Entity, With<VisualizationObject>>,
) {
    // If we don't have pending updated game state from WASM, abort early
    let Some(updated_game_state) = UPDATED_GAME_STATE.get() else {
        return;
    };

    let Some(new_game_state) = updated_game_state.lock().unwrap().take() else {
        return;
    };

    // We have new game state - blow away all of our current state
    for entity in &entity_query {
        commands.entity(entity).despawn();
    }

    *game_state = new_game_state;

    setup_after_reload_game_log(commands, meshes, materials, game_state);
}

fn setup_after_reload_game_log(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_state: ResMut<GameState>
) {
    // This function assumes we're setting up state from tick zero
    // Maybe there's a world where you can live-watch matches and want to join in some intermediate state, but that's not this world
    assert!(game_state.current_tick == 0);
    let game_log = game_state.game_log.as_ref().unwrap();
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
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(mesh_shape)),
                        material: materials.add(color),
                        transform,
                        ..default()
                    },
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
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Circle { radius: 0.5 })), // ZJ-TODO: read radius from ball object
                        material: materials.add(Color::linear_rgb(1.0, 0.0, 0.0)),
                        transform,
                        ..default()
                    },
                ));
            },
            SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                let translation = Vec3::new(position.x, position.z, position.y);
                let transform = Transform {
                    translation: translation.clone(),
                    rotation: Quat::default(),
                    scale: Vec3::ONE, // ZJ-TODO
                };
                commands.spawn((
                    VisualizationObject,
                    CombatantVisualizer { id: *combatant_id, desired_location: translation, last_position: translation },
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Capsule2d::new(1.0, 2.0))), // ZJ-TODO: read radius
                        material: materials.add(Color::linear_rgb(0.0, 1.0, 0.0)),
                        transform,
                        ..default()
                    },
                ));
            },
            _ => {}, // ZJ-TODO: we should assert if we have any unexpected events
        }
    }
}

fn update(
    mut game_state: ResMut<GameState>,
    mut combatants_query: Query<(&mut CombatantVisualizer, &mut Transform), Without<BallVisualizer>>,
    mut balls_query: Query<(&mut BallVisualizer, &mut Transform), Without<CombatantVisualizer>>,
    timer: Res<Time>,
) {
    if !matches!(game_state.mode, VisualizationMode::Play) {
        // Only play is support at the moment
        // ZJ-TODO: add input checking to switch modes
        return;
    }

    // Only update the simulation every second, otherwise would be too fast
    // ZJ-TODO: allow this to be configurable
    const TICKS_PER_SECOND: u64 = 10;
    const TIME_BETWEEN_TICKS_MILLIS: u64 = 1000 / TICKS_PER_SECOND;
    const TIME_BETWEEN_TICKS: Duration = Duration::from_millis(TIME_BETWEEN_TICKS_MILLIS);
    let time_since_last_update = Instant::now() - game_state.last_update_time;
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

    game_state.current_tick += 1;

    let current_tick = game_state.current_tick;
    let events_this_tick = 
    {
        let game_log = game_state.game_log.as_ref().unwrap();
        game_log.ticks().iter().nth(current_tick as usize).unwrap()
    };
        
    for event in &events_this_tick.simulation_events {
        match event {
            SimulationEvent::ArenaObjectPositionUpdate { .. } => { /* no-op, nothing to move with arena objects currently - this may change if plates start moving */},
            SimulationEvent::BallPositionUpdate { ball_id, position } => {
                let (mut ball_vis, _) = balls_query.iter_mut()
                    .filter(|(ball_vis, _)| ball_vis.id == *ball_id)
                    .next()
                    .unwrap();

                ball_vis.last_position = ball_vis.desired_location;
                ball_vis.desired_location = Vec3::new(position.x, position.z, position.y);
            },
            SimulationEvent::CombatantPickedUpBall { combatant_id, ball_id } => {
                // ZJ-TODO
            },
            SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                let (mut combatant_vis, _) = combatants_query.iter_mut()
                    .filter(|(combatant_vis, _)| combatant_vis.id == *combatant_id)
                    .next()
                    .unwrap();

                combatant_vis.last_position = combatant_vis.desired_location;
                combatant_vis.desired_location = Vec3::new(position.x, position.z, position.y);
            },
            SimulationEvent::CombatantOnPlate { combatant_id, plate_id } => {},
            SimulationEvent::CombatantOffPlate { combatant_id, plate_id } => {},
            SimulationEvent::BallThrownAtEnemy { thrower_id, enemy_id, ball_id, ball_impulse_vector } => {},
            SimulationEvent::BallThrownAtTeammate { thrower_id, teammate_id, ball_id, ball_impulse_vector } => {},
            SimulationEvent::BallCollisionEnemy { thrower_id, enemy_id, ball_id } => {},
            SimulationEvent::BallCollisionArena { thrower_id, original_target_id, ball_id } => {},
            SimulationEvent::BallExplosion { ball_id, charge } => {},
            SimulationEvent::BallExplosionForceApplied { ball_id, combatant_id, force_magnitude, force_direction } => {},
            SimulationEvent::PointsScoredByCombatant { plate_id, combatant_id, points } => {},
        }
    }

    game_state.last_update_time = Instant::now();
}

fn display_game_log_perf(
    game_state: Res<GameState>,
    mut text_query: Query<&mut Text, With<GameLogPerfText>>,
) {
    let mut text = text_query.get_single_mut().expect("failed to get debug position text component");
    let Some(ref game_log) = game_state.game_log else {
        return;
    };

    // Bevy default font doesn't display unicode (or at least 'μ')
    // Just replace with 'u'
    text.sections[0].value = game_log.perf_string().replace("μ", "u") + format!(" (tick {})", game_state.current_tick).as_str();
}


fn display_mouse_hover(
    mut camera_query: Query<(&Camera, &GlobalTransform)>,
    vis_objects_query: Query<&Transform, Or<(With<CombatantVisualizer>, With<BallVisualizer>)>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut text_query: Query<&mut Text, With<DebugPositionText>>,
) {   
    let mut text = text_query.get_single_mut().expect("failed to get debug position text component");
    text.sections[0].value = format!("");

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
    let Some(world_pos) =
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

        text.sections[0].value = format!("({}, {}, {})",
            vis_object_transform.translation.x.round(),
            vis_object_transform.translation.y.round(),
            vis_object_transform.translation.z.round(),
        );
        return;
    }
}
