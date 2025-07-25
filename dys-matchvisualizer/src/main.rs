use std::sync::{Arc, Mutex};

use dys_simulation::{game_log::GameLog, simulation::simulation_event::SimulationEvent};
use dys_stat::combatant_statline::CombatantStatline;

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

mod ui;
mod visualizer;

use crate::ui::{components as UiComponents, UiSystems};
use crate::visualizer::components::*;
use crate::visualizer::resources::*;
use crate::visualizer::VisualizerSystems;

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

#[derive(Component, Clone)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
}

impl AnimationConfig {
    fn new(first: usize, last: usize) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
        }
    }
}

/// This is quite the hack.
///
/// Once we start the Bevy app, we don't have a handle to the running process any more,
/// and can't update the VisualizationState resource in the standard Bevy ways.
///
/// To get around this, we have this static OnceCell, that holds an Arc<Mutex<Option<VisualizationState>>>.
static UPDATED_VIS_STATE: OnceCell<Arc<Mutex<Option<VisualizationState>>>> = OnceCell::new();

const FONT_FILE: &str = "fonts/Teko-Medium.ttf";

#[wasm_bindgen(js_name = initializeWithCanvas)]
pub fn initialize_with_canvas(
    canvas_id: String
) {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("DAX Match Visualizer"),
                        name: Some(String::from("DAX Match Visualizer")),
                        canvas: if canvas_id.is_empty() { None } else { Some(canvas_id) },
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
                })
                .set(ImagePlugin::default_nearest()),
        ))
        .add_plugins((
            ui::UiPlugin,
            visualizer::VisualizerPlugin,
        ))
        .configure_sets(
            Update,
            // Ensure that the visualizer runs before all other systems
            (
                VisualizerSystems,
                UiSystems,
            ).chain()
        )
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
        ))
        .run();
}

fn restart_with_local_game_log() {
    #[cfg(not(target_family = "wasm"))]
    {
        let game_log_bytes =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/data/game_log.bin"))
                .unwrap();

        let world_state_bytes =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/data/world_state.bin"))
                .unwrap();

        load_game_log(
            game_log_bytes,
            world_state_bytes
        );
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
    updated_visualization_state.lock().unwrap().replace(VisualizationState::from(game_log, world));
}

#[wasm_bindgen]
pub fn exit() {
    UPDATED_VIS_STATE
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .replace(VisualizationState::default());
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -100.0, // Default sets this to zero, when it should be negative
            far: 1000.0,
            scaling_mode: ScalingMode::Fixed {
                width: 900.0,
                height: 900.0,
            },
            scale: 0.13,
            viewport_origin: Default::default(),
            area: Default::default(),
        }),
    ));
}

fn try_reload_vis_state(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut vis_state: ResMut<VisualizationState>,
    asset_server: Res<AssetServer>,
    entity_query: Query<Entity, Or<(With<VisualizationObject>, With<Text>)>>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
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

    setup_after_reload_game_log(commands, meshes, materials, asset_server, vis_state, texture_atlas_layouts);
}

fn setup_after_reload_game_log(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    vis_state: ResMut<VisualizationState>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // If we're in the exit state, do nothing
    if vis_state.should_exit {
        return;
    }

    // This function assumes we're setting up state from tick zero
    // Maybe there's a world where you can live-watch matches and want to join in some intermediate state, but that's not this world
    assert_eq!(vis_state.current_tick, 0);
    let game_log = vis_state.game_log.as_ref().unwrap();
    assert!(!game_log.ticks().is_empty());

    let tick_zero = game_log.ticks().iter().next().unwrap();
    assert_eq!(tick_zero.tick_number, 0);

    // Combatant sprites
    let combatant_idle_atlas_layout = TextureAtlasLayout::from_grid(
        UVec2::splat(64),
        4,
        1,
        None,
        None
    );
    let combatant_idle_texture_atlas_layout = texture_atlas_layouts.add(combatant_idle_atlas_layout);
    let combatant_idle_animation_config = AnimationConfig::new(0, 3);
    let mut combatant_idle_texture_atlas = TextureAtlas::from(combatant_idle_texture_atlas_layout);
    combatant_idle_texture_atlas.index = combatant_idle_animation_config.first_sprite_index;

    let mut combatant_idle_spritesheet = Sprite::from_atlas_image(
        asset_server.load("sprites/character-idle-wip.png"),
        combatant_idle_texture_atlas,
    );
    combatant_idle_spritesheet.custom_size = Some(Vec2::splat(6.0));

    let combatant_running_atlas_layout = TextureAtlasLayout::from_grid(
        UVec2::splat(64),
        8,
        1,
        None,
        None
    );
    let combatant_running_texture_atlas_layout = texture_atlas_layouts.add(combatant_running_atlas_layout);
    let combatant_running_animation_config = AnimationConfig::new(0, 7);
    let mut combatant_running_texture_atlas = TextureAtlas::from(combatant_running_texture_atlas_layout);
    combatant_running_texture_atlas.index = combatant_running_animation_config.first_sprite_index;

    let mut combatant_running_spritesheet = Sprite::from_atlas_image(
        asset_server.load("sprites/character-running-wip.png"),
        combatant_running_texture_atlas,
    );
    combatant_running_spritesheet.custom_size = Some(Vec2::splat(6.0));

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
            SimulationEvent::BallPositionUpdate { ball_id, position, charge: _ } => {
                // ZJ-TODO: adding 10.0 sucks, but is necessary for balls to show above combatants in z-ordering
                //          otherwise the ball can "hide" under combatants
                let translation = Vec3::new(position.x, position.z, position.y + 10.0);
                let transform = Transform {
                    translation,
                    rotation: Quat::default(),
                    scale: Vec3::ONE,
                };

                let default_ball_color = Color::linear_rgb(0.75, 0.75, 0.0);
                commands.spawn((
                    VisualizationObject,
                    BallVisualizer {
                        id: *ball_id,
                        desired_location: translation,
                        last_position: translation,
                        desired_scale: Vec3::ONE,
                        last_scale: Vec3::ONE,
                        desired_charge: 0.0,
                    },
                    Mesh2d(meshes.add(Circle { radius: 0.5 })), // ZJ-TODO: read radius from ball object
                    MeshMaterial2d(materials.add(default_ball_color)),
                    transform,
                ));
            },
            SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                let translation = Vec3::new(position.x, position.z, position.y);
                let transform = Transform {
                    translation,
                    rotation: Quat::default(),
                    scale: Vec3::ONE,
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

                name = format!("{} ({})",
                    name.splitn(2, " ").skip(1).collect::<String>(),
                    combatant_id
                );

                // let mut combatant_idle_spritesheet = combatant_idle_spritesheet.clone();
                // combatant_idle_spritesheet.color = Color::srgb(
                //     0.0,
                //     if home_team { 0.4 } else { 0.0 },
                //     if home_team { 0.0 } else { 1.0 },
                // );
                //
                // combatant_idle_spritesheet.flip_x = !home_team;

                let mut combatant_running_spritesheet = combatant_running_spritesheet.clone();
                combatant_running_spritesheet.color = Color::srgb(
                    if home_team { 0.0 } else { 0.8 },
                    if home_team { 0.4 } else { 0.8 },
                    if home_team { 0.0 } else { 0.8 },
                );

                combatant_running_spritesheet.flip_x = !home_team;

                commands.spawn((
                    VisualizationObject,
                    CombatantVisualizer {
                        id: *combatant_id,
                        instance_id,
                        desired_location: translation,
                        last_position: translation
                    },
                    // combatant_idle_spritesheet,
                    combatant_running_spritesheet,
                    // combatant_idle_animation_config.clone(),
                    combatant_running_animation_config.clone(),
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
                        UiComponents::CombatantIdText {
                            combatant_id: *combatant_id,
                            is_stunned: false,
                        },
                    ));
                });
            },
            _ => {}, // ZJ-TODO: we should assert if we have any unexpected events
        }
    }

    let combatant_statlines = CombatantStatline::from_game_log(game_log);
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
        UiComponents::PostgameScoreboard,
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
    mut combatants_query: Query<(&mut CombatantVisualizer, &mut Transform, &mut Sprite, &mut AnimationConfig), Without<BallVisualizer>>,
    mut combatant_id_text_query: Query<&mut UiComponents::CombatantIdText>,
    mut balls_query: Query<(&mut BallVisualizer, &mut Transform, &mut MeshMaterial2d<ColorMaterial>), Without<CombatantVisualizer>>,
    mut camera_query: Query<(&mut Transform, &mut Projection), (With<Camera2d>, Without<CombatantVisualizer>, Without<BallVisualizer>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut fx_query: Query<(Entity, &mut FxEntity, &mut Sprite, &mut AnimationConfig), Without<CombatantVisualizer>>,
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

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    // Visual updates can occur ever frame
    for (mut combatant_vis, mut combatant_transform, mut sprite, animation_config) in combatants_query.iter_mut() {
        // ZJ-TODO: track what direction a combatant is "facing" in case we allow backpedaling
        //          that's a long time from now though, so this works in the interim
        if (combatant_vis.desired_location.x - combatant_transform.translation.x).abs() > 0.5 {
            sprite.flip_x = combatant_vis.desired_location.x < combatant_transform.translation.x;
        }

        combatant_transform.translation = combatant_vis.last_position.lerp(
            combatant_vis.desired_location,
            lerp_progress
        );

        min_x = min_x.min(combatant_transform.translation.x);
        max_x = max_x.max(combatant_transform.translation.x);
        min_y = min_y.min(combatant_transform.translation.y);
        max_y = max_y.max(combatant_transform.translation.y);

        combatant_vis.last_position = combatant_transform.translation;
        debug!("Workaround for ignored field: {}", combatant_vis.instance_id);

        let Some(atlas) = &mut sprite.texture_atlas else {
            warn!("expected texture atlas for sprite!");
            continue;
        };

        atlas.index = vis_state.current_tick as usize % (animation_config.last_sprite_index + 1);
    }

    // ZJ-TODO: WIP camera zoom + movement
    //          this really needs lerping as it's kinda vomit inducing atm

    // let x_offset = (max_x + min_x - 100.0) * 0.2;
    // let y_offset = (max_y + min_y - 100.0) * 0.2;

    let x_offset = 0.0;
    let y_offset = 0.0;
    let x_base = -8.0;
    let y_base = -6.0;

    let (mut camera_transform, _) = camera_query.single_mut().unwrap();
    camera_transform.translation.x = x_offset + x_base;
    camera_transform.translation.y = y_offset + y_base;

    // let Projection::Orthographic(ref mut projection) = *projection else {
    //     panic!("expected orthographic projection");
    // };
    //
    // let max_dist = (max_x - min_x).max(max_y - min_y);
    // projection.scale = 0.13_f32.min(max_dist * 0.01);

    for (mut ball_vis, mut ball_transform, ball_mat) in balls_query.iter_mut() {
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

        let ball_color_mat = materials.get_mut(ball_mat.0.id()).unwrap();
        ball_color_mat.color = Color::linear_rgb(0.75, 0.75 - (ball_vis.desired_charge / 100.0), 0.0);
    }

    if time_since_last_update < TIME_BETWEEN_TICKS {
        return;
    }

    if matches!(vis_state.mode, VisualizationMode::Play) && !vis_state.end_of_game {
        vis_state.current_tick += 1;
    }

    for (entity, mut fx_entity, mut sprite, animation_config) in fx_query.iter_mut() {
        let Some(atlas) = &mut sprite.texture_atlas else {
            warn!("expected texture atlas for sprite!");
            continue;
        };

        atlas.index = fx_entity.current_lifespan_in_ticks;
        fx_entity.current_lifespan_in_ticks += 1;
        if fx_entity.current_lifespan_in_ticks > animation_config.last_sprite_index {
            commands.entity(entity).despawn();
        };
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
                SimulationEvent::BallPositionUpdate { ball_id, position, charge } => {
                    let (mut ball_vis, _, _) = balls_query.iter_mut()
                        .find(|(ball_vis, _, _)| ball_vis.id == *ball_id)
                        .unwrap();

                    ball_vis.desired_location = Vec3::new(position.x, position.z, position.y);
                    // Every 3 units vertically, make the ball twice as big
                    let scale_modifier = (1.0 + (position.y / 3.0)).max(1.0);
                    ball_vis.desired_scale = Vec3::ONE * scale_modifier;

                    ball_vis.desired_charge = *charge;
                },
                SimulationEvent::CombatantPositionUpdate { combatant_id, position } => {
                    let (mut combatant_vis, _, _, _) = combatants_query.iter_mut()
                        .find(|(combatant_vis, _, _, _)| combatant_vis.id == *combatant_id)
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
                    let (_, ball_pos, _) = balls_query.iter()
                        .find(|(ball_vis, _, _)| ball_vis.id == *ball_id)
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
                SimulationEvent::ThrownBallCaught { thrower_id: _, catcher_id: _, ball_id } => {
                    let (_, catch_pos, _) = balls_query.iter()
                        .find(|(ball_vis, _, _)| ball_vis.id == *ball_id)
                        .unwrap();

                    // Catch sprite
                    let atlas_layout = TextureAtlasLayout::from_grid(
                        UVec2::splat(64),
                        9,
                        1,
                        None,
                        None
                    );
                    let texture_atlas_layout = texture_atlas_layouts.add(atlas_layout);
                    let catch_fx_animation_config = AnimationConfig::new(0, 8);
                    let mut texture_atlas = TextureAtlas::from(texture_atlas_layout);
                    texture_atlas.index = catch_fx_animation_config.first_sprite_index;

                    let mut catch_fx_spritesheet = Sprite::from_atlas_image(
                        asset_server.load("sprites/catch-fx-wip.png"),
                        texture_atlas,
                    );
                    catch_fx_spritesheet.custom_size = Some(Vec2::splat(16.0));

                    commands.spawn((
                        FxEntity::default(),
                        catch_fx_spritesheet,
                        catch_fx_animation_config,
                        catch_pos.to_owned(),
                    ));
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
    mut text_query: Query<&mut Text2d, With<UiComponents::GameLogPerfText>>,
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
        Query<(&mut Text2d, &mut TextColor), With<UiComponents::HomeTeamScoreText>>,
        Query<(&mut Text2d, &mut TextColor), With<UiComponents::AwayTeamScoreText>>,
        Query<&mut Text2d, With<UiComponents::MatchTimerText>>,
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
        (&mut Text2d, &mut TextColor, &mut Node, Has<UiComponents::HomeTeamScoreUpdateText>),
        Or<(With<UiComponents::HomeTeamScoreUpdateText>, With<UiComponents::AwayTeamScoreUpdateText>)>
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
    mut combatants_query: Query<(&mut TextColor, &UiComponents::CombatantIdText)>,
) {
    for (mut text_color, combatant_id_text) in combatants_query.iter_mut() {
        if combatant_id_text.is_stunned {
            *text_color = TextColor(Color::srgb(1.0, 0.0, 0.0));
        } else {
            *text_color = TextColor(Color::WHITE);
        }
    }
}