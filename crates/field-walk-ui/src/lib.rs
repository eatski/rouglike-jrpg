mod bounce;
mod camera;
pub mod coast_lookup;
mod components;
pub mod constants;
mod encounter;
mod events;
mod execute;
mod input;
pub mod map_mode;
pub mod rendering;
mod resources;
pub mod simple_tiles;
mod smooth_move;
pub mod tile_pool;

// world-ui から統合されたモジュール
mod field_smooth_move;
pub mod exploration_data;
mod hud;
mod minimap;
mod player_input;
pub mod field_rendering;
mod tile_action;

pub use bounce::{start_bounce, update_bounce, Bounce};
pub use camera::{camera_follow, setup_camera};
pub use components::{MovementLocked, PendingMove};
pub use constants::*;
pub use encounter::check_encounter_system;
pub use events::{MovementBlockedEvent, PlayerMovedEvent, TileEnteredEvent};
pub use execute::{execute_move, ExecuteMoveResult};
pub use input::{process_movement_input, MovementInput};
pub use map_mode::{reset_map_mode_system, toggle_map_mode_system, MapModeState, NORMAL_ZOOM};
pub use rendering::{load_tile_textures, spawn_boat_entities, BoatSpawnsResource, BossCaveWorldPos, TileTextures};
pub use resources::MovementState;
pub use simple_tiles::{update_simple_tiles, SimpleTile, SimpleTileMap};
pub use smooth_move::{
    ease_out_quad, is_smooth_moving, start_smooth_move, update_smooth_move, SmoothMove,
    MOVE_DURATION,
};
pub use terrain::MoveResult;
pub use tile_pool::{create_tile_pool, init_tile_pool, update_visible_tiles, PooledTile, TilePool};

// world-ui から統合された再エクスポート
pub use hud::{cleanup_hud, setup_hud, toggle_hud_visibility, update_hud};
pub use exploration_data::{init_exploration_system, update_exploration_system, ExplorationData};
pub use minimap::{init_minimap_system, toggle_minimap_visibility_system, update_minimap_texture_system};
pub use player_input::{player_movement, sync_boat_with_player};
pub use field_rendering::{spawn_field_map, spawn_field_map_with_rng, spawn_player};
pub use field_smooth_move::handle_field_move_completed;
pub use tile_action::check_tile_action_system;
pub use field_rendering::SpawnPosition;

use bevy::prelude::*;
use app_state::{BattleState, InField, SceneState};
use field_core::{ActiveMap, Player, TilePosition};

/// フィールド離脱時にプレイヤーの移動関連コンポーネントと状態をクリーンアップする。
/// OnExit(InField) で呼ばれ、戦闘開始・町入場・祠入場時のクリーンアップを一元化する。
pub fn cleanup_player_movement(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    mut move_state: ResMut<MovementState>,
) {
    if let Ok(entity) = player_query.single() {
        commands
            .entity(entity)
            .remove::<MovementLocked>()
            .remove::<SmoothMove>()
            .remove::<PendingMove>()
            .remove::<Bounce>();
    }
    *move_state = MovementState::default();
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovementBlockedEvent>()
            .add_message::<PlayerMovedEvent>()
            .add_message::<TileEnteredEvent>()
            .init_resource::<MovementState>()
            .add_systems(PostUpdate, sync_tile_to_transform);
    }
}

/// TilePosition変更時にTransformを自動同期するシステム。
/// SmoothMoveアニメーション中はスキップされる（SmoothMove側がTransformを制御するため）。
#[allow(clippy::type_complexity)]
fn sync_tile_to_transform(
    active_map: Option<Res<ActiveMap>>,
    mut query: Query<
        (&TilePosition, &mut Transform),
        (Changed<TilePosition>, With<Player>, Without<SmoothMove>),
    >,
) {
    let Some(active_map) = active_map else {
        return;
    };
    for (tile_pos, mut transform) in &mut query {
        let (world_x, world_y) = active_map.to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }
}

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
