use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};

use app_state::{BattleState, InField, SceneState};
use battle_ui::{field_menu_display_system, field_menu_input_system};
use app_state::OpenedChests;
use cave_ui::{
    cave_message_display_system, cave_message_input_system, cave_player_movement,
    check_chest_system, check_ladder_system, despawn_cave_entities, restore_field_from_cave,
    setup_cave_scene, start_cave_smooth_move, update_cave_smooth_move, update_cave_tiles,
};
use movement_ui::{
    start_bounce, update_bounce, MovementBlockedEvent, PlayerArrivedEvent, PlayerMovedEvent,
    TileEnteredEvent,
};
use app_state::PartyState;
use movement_ui::{MovementState, WINDOW_SIZE};
use hokora_ui::{
    cleanup_hokora_scene, hokora_display_system, hokora_input_system, setup_hokora_scene,
};
use town_ui::{cleanup_town_scene, setup_town_scene, town_display_system, town_input_system};
use world_ui::{
    camera_follow, check_encounter_system, cleanup_hud, init_exploration_system,
    init_minimap_system, init_tile_pool, setup_camera, setup_hud,
    spawn_field_map, spawn_player,
    toggle_hud_visibility, update_hud, MapModeState,
};

fn main() {
    let title = std::env::args()
        .nth(1)
        .map(|name| format!("Roguelike JRPG [{}]", name))
        .unwrap_or_else(|| "Roguelike JRPG".to_string());

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title,
                    resolution: WindowResolution::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    )
    .init_state::<SceneState>()
    .init_state::<BattleState>()
    .add_computed_state::<InField>()
    .add_message::<MovementBlockedEvent>()
    .add_message::<PlayerMovedEvent>()
    .add_message::<PlayerArrivedEvent>()
    .add_message::<TileEnteredEvent>()
    .init_resource::<MovementState>()
    .init_resource::<MapModeState>()
    .init_resource::<PartyState>()
    .init_resource::<OpenedChests>()
    .add_systems(
        Startup,
        (
            spawn_field_map,
            setup_camera,
            spawn_player,
            init_tile_pool,
            init_exploration_system,
            init_minimap_system,
        )
            .chain(),
    )
    // HUD: InFieldで自動管理
    .add_systems(OnEnter(InField), setup_hud)
    .add_systems(
        Update,
        (toggle_hud_visibility, update_hud)
            .chain()
            .run_if(in_state(InField)),
    )
    .add_systems(
        Update,
        (field_menu_input_system, field_menu_display_system)
            .chain()
            .run_if(in_state(InField)),
    )
    .add_systems(OnExit(InField), cleanup_hud);

    // Exploring
    world_ui::register_exploring_all_systems(&mut app);

    // Battle
    battle_ui::register_battle_all_systems(&mut app);

    app
    // Town
    .add_systems(OnEnter(SceneState::Town), setup_town_scene)
    .add_systems(
        Update,
        (town_input_system, town_display_system)
            .chain()
            .run_if(in_state(SceneState::Town)),
    )
    .add_systems(OnExit(SceneState::Town), cleanup_town_scene)
    // Hokora
    .add_systems(OnEnter(SceneState::Hokora), setup_hokora_scene)
    .add_systems(
        Update,
        (hokora_input_system, hokora_display_system)
            .chain()
            .run_if(in_state(SceneState::Hokora)),
    )
    .add_systems(OnExit(SceneState::Hokora), cleanup_hokora_scene)
    // Cave
    .add_systems(OnEnter(SceneState::Cave), setup_cave_scene)
    .add_systems(
        Update,
        (
            cave_player_movement,
            start_cave_smooth_move,
            ApplyDeferred,
            update_cave_smooth_move,
            start_bounce,
            update_bounce,
            update_cave_tiles,
            camera_follow,
            check_chest_system,
            cave_message_input_system,
            cave_message_display_system,
            check_ladder_system,
            check_encounter_system,
        )
            .chain()
            .run_if(in_state(SceneState::Cave).and(in_state(BattleState::None))),
    )
    .add_systems(OnExit(SceneState::Cave), (despawn_cave_entities, restore_field_from_cave).chain());

    app.run();
}
