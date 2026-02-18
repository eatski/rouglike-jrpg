use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};

use app_state::{BattleState, InField, SceneState};
use battle_ui::{
    battle_blink_system, battle_display_system, battle_input_system, battle_shake_system,
    cleanup_battle_scene, field_spell_display_system, field_spell_input_system,
    setup_battle_scene,
};
use cave_ui::{
    cave_player_movement, check_ladder_system, cleanup_cave_scene, setup_cave_scene,
    start_cave_smooth_move, update_cave_smooth_move, update_cave_tiles,
};
use movement_ui::{
    start_bounce, update_bounce, MovementBlockedEvent, PlayerArrivedEvent, PlayerMovedEvent,
    TileEnteredEvent,
};
use app_state::{FieldSpellMenuOpen, PartyState};
use movement_ui::{MovementState, WINDOW_SIZE};
use town_ui::{cleanup_town_scene, setup_town_scene, town_display_system, town_input_system};
use world_ui::{
    camera_follow, check_encounter_system, check_tile_action_system, cleanup_hud, init_exploration_system,
    init_minimap_system, init_tile_pool, player_movement, setup_camera, setup_hud,
    spawn_field_map, spawn_player, start_smooth_move, sync_boat_with_player,
    toggle_hud_visibility, toggle_map_mode_system, toggle_minimap_visibility_system,
    update_exploration_system, update_hud, update_minimap_texture_system, update_smooth_move,
    update_visible_tiles, MapModeState,
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
    .init_resource::<FieldSpellMenuOpen>()
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
        (field_spell_input_system, field_spell_display_system)
            .chain()
            .run_if(in_state(InField)),
    )
    .add_systems(OnExit(InField), cleanup_hud)
    // Exploring
    .add_systems(
        Update,
        (
            toggle_map_mode_system,
            toggle_minimap_visibility_system,
            player_movement,
            start_bounce,
            start_smooth_move,
            ApplyDeferred,
            update_smooth_move,
            update_bounce,
            update_visible_tiles,
            update_exploration_system,
            update_minimap_texture_system,
            sync_boat_with_player,
            camera_follow,
            check_tile_action_system,
            check_encounter_system,
        )
            .chain()
            .run_if(in_state(SceneState::Exploring).and(in_state(BattleState::None))),
    )
    // Battle
    .add_systems(OnEnter(BattleState::Active), setup_battle_scene)
    .add_systems(
        Update,
        (
            battle_input_system,
            battle_display_system,
            battle_blink_system,
            battle_shake_system,
        )
            .chain()
            .run_if(in_state(BattleState::Active)),
    )
    .add_systems(OnExit(BattleState::Active), cleanup_battle_scene)
    // Town
    .add_systems(OnEnter(SceneState::Town), setup_town_scene)
    .add_systems(
        Update,
        (town_input_system, town_display_system)
            .chain()
            .run_if(in_state(SceneState::Town)),
    )
    .add_systems(OnExit(SceneState::Town), cleanup_town_scene)
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
            check_ladder_system,
            check_encounter_system,
        )
            .chain()
            .run_if(in_state(SceneState::Cave).and(in_state(BattleState::None))),
    )
    .add_systems(OnExit(SceneState::Cave), cleanup_cave_scene);

    app.run();
}
