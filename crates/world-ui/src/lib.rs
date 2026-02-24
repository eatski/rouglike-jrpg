mod hud;
pub mod map_mode;
mod minimap;
mod player_input;
pub mod rendering;
pub mod resources;
pub mod smooth_move;
pub mod tile_action;

use bevy::prelude::*;
use app_state::{BattleState, InField, SceneState};
use field_walk_ui::{start_bounce, start_smooth_move, update_bounce, update_smooth_move};

// field-walk-ui からの再エクスポート
pub use field_walk_ui::{
    camera_follow, check_encounter_system, setup_camera,
    reset_map_mode_system, toggle_map_mode_system, MapModeState, NORMAL_ZOOM,
    load_tile_textures, spawn_boat_entities, BoatSpawnsResource, BossCaveWorldPos, TileTextures,
    create_tile_pool, init_tile_pool, update_visible_tiles, PooledTile, TilePool,
};

pub use hud::{cleanup_hud, setup_hud, toggle_hud_visibility, update_hud};
pub use map_mode::{
    init_exploration_system,
    update_exploration_system, ExplorationData,
};
pub use minimap::{init_minimap_system, toggle_minimap_visibility_system, update_minimap_texture_system};
pub use player_input::{player_movement, sync_boat_with_player};
pub use rendering::{spawn_field_map, spawn_field_map_with_rng, spawn_player};
pub use smooth_move::handle_field_move_completed;
pub use tile_action::check_tile_action_system;
pub use resources::SpawnPosition;

/// 移動コアシステム（エンカウント・タイルアクション除く）
///
/// toggle_map_mode_systemはCamera2dをクエリするが、if let Ok(...) guardで
/// MinimalPlugins環境でも安全にスキップされる。
pub fn register_exploring_movement_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            toggle_map_mode_system,
            player_movement,
            start_bounce,
            start_smooth_move,
            ApplyDeferred,
            update_smooth_move,
            handle_field_move_completed,
            update_bounce,
            sync_boat_with_player,
        )
            .chain()
            .run_if(in_state(SceneState::Exploring).and(in_state(BattleState::None))),
    );
}

/// タイルアクション + エンカウント
pub fn register_exploring_event_systems(app: &mut App) {
    app.add_systems(
        Update,
        (check_tile_action_system, check_encounter_system)
            .chain()
            .after(sync_boat_with_player)
            .run_if(in_state(SceneState::Exploring).and(in_state(BattleState::None))),
    );
}

/// 全ロジックシステム（テスト用: レンダリング非依存のみ）
pub fn register_exploring_logic_systems(app: &mut App) {
    register_exploring_movement_systems(app);
    register_exploring_event_systems(app);
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapModeState>()
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
            .add_systems(OnEnter(InField), setup_hud)
            .add_systems(
                Update,
                (toggle_hud_visibility, update_hud)
                    .chain()
                    .run_if(in_state(InField)),
            )
            .add_systems(OnExit(InField), cleanup_hud);

        register_exploring_all_systems(app);
    }
}

/// 全システム（本番用: レンダリング依存含む）
pub fn register_exploring_all_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            toggle_map_mode_system,
            toggle_minimap_visibility_system,
            player_movement,
            start_bounce,
            start_smooth_move,
            ApplyDeferred,
            update_smooth_move,
            handle_field_move_completed,
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
    );
}
