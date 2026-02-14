use bevy::camera::Projection;
use bevy::prelude::*;

use world::exploration::{ExplorationMap, VIEW_RADIUS};
use terrain::{MAP_HEIGHT, MAP_WIDTH};

use movement_ui::{Player, PlayerMovedEvent, TilePosition};
use shared_ui::{MAP_PIXEL_WIDTH, VISIBLE_SIZE};

use crate::resources::SpawnPosition;

/// マップモードのズーム値（マップ全体表示用）
pub const MAP_MODE_ZOOM: f32 = MAP_PIXEL_WIDTH;
/// 通常時のズーム値
pub const NORMAL_ZOOM: f32 = VISIBLE_SIZE;

/// マップモードの状態を管理するリソース
#[derive(Resource, Default)]
pub struct MapModeState {
    pub enabled: bool,
}

/// 探索データを管理するリソース
#[derive(Resource)]
pub struct ExplorationData {
    pub map: ExplorationMap,
}

/// マップ生成時にExplorationDataを初期化するシステム
pub fn init_exploration_system(mut commands: Commands, spawn_pos: Res<SpawnPosition>) {
    let mut exploration_map = ExplorationMap::new(MAP_WIDTH, MAP_HEIGHT);

    // プレイヤーの初期位置周辺を探索済みにする
    exploration_map.update_visibility(spawn_pos.x, spawn_pos.y, VIEW_RADIUS);

    commands.insert_resource(ExplorationData {
        map: exploration_map,
    });
}

/// プレイヤー移動時に視界を更新するシステム
pub fn update_exploration_system(
    mut exploration_data: ResMut<ExplorationData>,
    player_query: Query<&TilePosition, With<Player>>,
    mut moved_events: MessageReader<PlayerMovedEvent>,
) {
    for _event in moved_events.read() {
        if let Ok(tile_pos) = player_query.single() {
            exploration_data
                .map
                .update_visibility(tile_pos.x, tile_pos.y, VIEW_RADIUS);
        }
    }
}

/// Mキー押下でマップモードをトグルするシステム
pub fn toggle_map_mode_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut map_mode_state: ResMut<MapModeState>,
    mut camera_query: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
) {
    if input_ui::is_map_toggle_just_pressed(&keyboard) {
        map_mode_state.enabled = !map_mode_state.enabled;

        // カメラズームと位置を変更
        if let Ok((mut projection, mut transform)) = camera_query.single_mut()
            && let Projection::Orthographic(ref mut ortho) = *projection
        {
            if map_mode_state.enabled {
                // マップモード: ズームアウト＆ワールド中心に固定
                ortho.scaling_mode = bevy::camera::ScalingMode::Fixed {
                    width: MAP_MODE_ZOOM,
                    height: MAP_MODE_ZOOM,
                };
                transform.translation.x = 0.0;
                transform.translation.y = 0.0;
            } else {
                // 通常モード: 元に戻す（カメラ位置はcamera_followで追従）
                ortho.scaling_mode = bevy::camera::ScalingMode::Fixed {
                    width: NORMAL_ZOOM,
                    height: NORMAL_ZOOM,
                };
            }
        }
    }
}
