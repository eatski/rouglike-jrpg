use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};

use ui::constants::WINDOW_SIZE;
use ui::events::{MovementBlockedEvent, PlayerMovedEvent};
use ui::resources::MovementState;
use ui::{
    battle_blink_system, battle_display_system, battle_input_system, battle_shake_system,
    camera_follow, check_encounter_system, cleanup_battle_scene, clear_virtual_input,
    init_exploration_system, init_minimap_system, init_tile_pool, manual_screenshot_system,
    player_movement, read_remote_commands, remote_screenshot_system, setup_battle_scene,
    setup_camera, spawn_field_map, spawn_player, start_bounce, start_smooth_move,
    sync_boat_with_player, toggle_map_mode_system, toggle_minimap_visibility_system,
    update_bounce, update_exploration_system, update_minimap_texture_system, update_smooth_move,
    update_visible_tiles, write_game_state_log, AppState, MapModeState, RemoteControlMode,
    VirtualInput,
};

fn main() {
    let remote_mode = std::env::args().any(|arg| arg == "--remote");

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Roguelike JRPG".to_string(),
                    resolution: WindowResolution::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    )
    .init_state::<AppState>()
    .add_message::<MovementBlockedEvent>()
    .add_message::<PlayerMovedEvent>()
    .init_resource::<MovementState>()
    .init_resource::<MapModeState>()
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
            check_encounter_system,
        )
            .chain()
            .run_if(in_state(AppState::Exploring)),
    )
    .add_systems(OnEnter(AppState::Battle), setup_battle_scene)
    .add_systems(
        Update,
        (
            battle_input_system,
            battle_display_system,
            battle_blink_system,
            battle_shake_system,
        )
            .chain()
            .run_if(in_state(AppState::Battle)),
    )
    .add_systems(OnExit(AppState::Battle), cleanup_battle_scene)
    .add_systems(Update, manual_screenshot_system);

    if remote_mode {
        // remote/ ディレクトリを初期化（両ファイルともセッション開始時にリセット）
        std::fs::create_dir_all("remote").ok();
        std::fs::write("remote/commands.jsonl", "").ok();
        std::fs::write("remote/response.jsonl", "").ok();

        app.insert_resource(RemoteControlMode::new())
            .init_resource::<VirtualInput>()
            .add_systems(PreUpdate, read_remote_commands)
            .add_systems(
                PostUpdate,
                (write_game_state_log, remote_screenshot_system),
            )
            .add_systems(Last, clear_virtual_input);
    }

    app.run();
}
