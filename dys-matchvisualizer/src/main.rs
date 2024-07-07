use dys_game::{game_log::GameLog, game_objects::{ball::BallId, combatant::CombatantId}, game_tick::GameTickNumber, simulation::simulation_event::SimulationEvent};

use bevy::{prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}};
use wasm_bindgen::prelude::wasm_bindgen;
use web_time::{Duration, Instant};

fn main() {
    // Intentionally empty for WASM.
    //
    // Web components with call `initialize_with_game_log` to start a visualization
    // with the game log they desire.

    #[cfg(not(target_family="wasm"))]
    {
        let game_log_bytes = include_bytes!("../data/game_log.bin");
        initialize_with_game_log(String::new(), game_log_bytes.to_vec());
    }
}

enum VisualizationMode {
    /// THe visualization will not progress
    Paused,

    /// The visualization will progress only when the viewer instructs it to
    Step,

    /// The visualization will progress without user intervention
    Play,
}

#[derive(Resource)]
struct GameState {
    game_log: GameLog,
    current_tick: GameTickNumber,
    last_update_time: Instant,
    mode: VisualizationMode,
}

#[derive(Component)]
struct CombatantVisualizer {
    pub id: CombatantId,
}

#[derive(Component)]
struct BallVisualizer {
    pub id: BallId,
}

#[wasm_bindgen]
pub fn initialize_with_game_log(
    canvas_id: String,
    serialized_game_log: Vec<u8>,
) {
    // ZJ-TODO: gracefully handle failures in parsing
    let game_log: GameLog = postcard::from_bytes(&serialized_game_log).unwrap();
    let canvas: Option<String> = if canvas_id.is_empty() { None } else { Some(canvas_id) };

    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas,
                    ..default()
                }),
                ..default()
            })
        )
        .insert_resource(GameState {
            game_log,
            current_tick: 0,
            last_update_time: Instant::now(),
            mode: VisualizationMode::Play, // ZJ-TODO: should start paused?
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_state: ResMut<GameState>
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            near: -100.0, // Default sets this to zero, when it should be negative
            far: 1000.0,
            scale: 0.33,
            ..default()
        },
        ..default()
    });

    // This function assumes we're setting up state from tick zero
    // Maybe there's a world where you can live-watch matches and want to join in some intermediate state, but that's not this world
    assert!(game_state.current_tick == 0);
    assert!(!game_state.game_log.ticks().is_empty());

    let tick_zero = game_state.game_log.ticks().iter().next().unwrap();
    assert!(tick_zero.tick_number == 0);

    for evt in &tick_zero.simulation_events {
        match evt {
            SimulationEvent::ArenaObjectPositionUpdate {  } => { /* ZJ-TODO */ },
            SimulationEvent::BallPositionUpdate { ball_id, position } => {
                let transform = Transform {
                    translation: Vec3::new(position.x, position.z, position.y),
                    rotation: Quat::default(),
                    scale: Vec3::ONE, // ZJ-TODO
                };
                commands.spawn((
                    BallVisualizer { id: *ball_id },
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Circle { radius: 2.0 })), // ZJ-TODO: read radius
                        material: materials.add(Color::linear_rgb(1.0, 0.0, 0.0)),
                        transform,
                        ..default()
                    },
                ));
            },
            SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                let transform = Transform {
                    translation: Vec3::new(position.x, position.z, position.y),
                    rotation: Quat::default(),
                    scale: Vec3::ONE, // ZJ-TODO
                };
                commands.spawn((
                    CombatantVisualizer { id: *combatant_id },
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Capsule2d::new(4.0, 8.0))), // ZJ-TODO: read radius
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
    mut combatants_query: Query<(&CombatantVisualizer, &mut Transform), Without<BallVisualizer>>,
    mut balls_query: Query<(&BallVisualizer, &mut Transform), Without<CombatantVisualizer>>,
) {
    if !matches!(game_state.mode, VisualizationMode::Play) {
        // Only play is support at the moment
        // ZJ-TODO: add input checking to switch modes
        return;
    }

    // Only update the simulation every second, otherwise would be too fast
    // ZJ-TODO: allow this to be configurable
    const TIME_BETWEEN_TICKS: Duration = Duration::from_millis(100);
    if (Instant::now() - game_state.last_update_time) < TIME_BETWEEN_TICKS {
        return;
    }

    game_state.current_tick += 1;

    let current_tick = game_state.current_tick;
    let events_this_tick = game_state.game_log.ticks().iter().nth(current_tick as usize).unwrap();
    for event in &events_this_tick.simulation_events {
        match event {
            SimulationEvent::ArenaObjectPositionUpdate {  } => {},
            SimulationEvent::BallPositionUpdate { ball_id, position } => {
                let (_, mut ball_transform) = balls_query.iter_mut()
                    .filter(|(ball_vis, _)| ball_vis.id == *ball_id)
                    .next()
                    .unwrap();

                ball_transform.translation = Vec3::new(position.x, position.z, position.y);
            },
            SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                let (_, mut combatant_transform) = combatants_query.iter_mut()
                    .filter(|(combatant_vis, _)| combatant_vis.id == *combatant_id)
                    .next()
                    .unwrap();

                combatant_transform.translation = Vec3::new(position.x, position.z, position.y);
            },
            SimulationEvent::CombatantOnPlate { combatant_id, plate_id } => {},
            SimulationEvent::CombatantOffPlate { combatant_id, plate_id } => {},
            SimulationEvent::BallThrownAtEnemy { thrower_id, enemy_id, ball_id } => {},
            SimulationEvent::BallThrownAtTeammate { thrower_id, teammate_id, ball_id } => {},
            SimulationEvent::BallCollisionEnemy { thrower_id, enemy_id, ball_id } => {},
            SimulationEvent::BallCollisionArena { thrower_id, original_target_id, ball_id } => {},
            SimulationEvent::BallExplosion { ball_id, charge } => {},
            SimulationEvent::BallExplosionForceApplied { ball_id, combatant_id, force_magnitude, force_direction } => {},
            SimulationEvent::PointsScoredByCombatant { plate_id, combatant_id, points } => {},
        }
    }

    game_state.last_update_time = Instant::now();
}