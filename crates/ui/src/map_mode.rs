use bevy::camera::Projection;
use bevy::prelude::*;

use game::exploration::{ExplorationMap, TileVisibility, VIEW_RADIUS};
use game::map::{MAP_HEIGHT, MAP_WIDTH};

use crate::components::{MapTile, Player, TilePosition};
use crate::constants::{MAP_PIXEL_WIDTH, VISIBLE_SIZE};
use crate::events::PlayerMovedEvent;
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

/// タイルの元の色を保存するコンポーネント
#[derive(Component)]
pub struct OriginalColor(pub Color);

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
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        map_mode_state.enabled = !map_mode_state.enabled;

        // カメラズームを変更
        if let Ok(mut projection) = camera_query.single_mut()
            && let Projection::Orthographic(ref mut ortho) = *projection
        {
            if map_mode_state.enabled {
                // マップモード: ズームアウト
                ortho.scaling_mode = bevy::camera::ScalingMode::Fixed {
                    width: MAP_MODE_ZOOM,
                    height: MAP_MODE_ZOOM,
                };
            } else {
                // 通常モード: 元に戻す
                ortho.scaling_mode = bevy::camera::ScalingMode::Fixed {
                    width: NORMAL_ZOOM,
                    height: NORMAL_ZOOM,
                };
            }
        }
    }
}

/// マップモード時にFog of Warを適用するシステム
pub fn apply_map_mode_fog_system(
    mut commands: Commands,
    map_mode_state: Res<MapModeState>,
    exploration_data: Res<ExplorationData>,
    mut tile_query: Query<(Entity, &TilePosition, &mut Sprite, Option<&OriginalColor>), With<MapTile>>,
) {
    if !map_mode_state.enabled {
        return;
    }

    // マップモードに切り替わった瞬間のみ元の色を保存
    if map_mode_state.is_changed() && map_mode_state.enabled {
        for (entity, _tile_pos, sprite, original_color) in tile_query.iter() {
            if original_color.is_none() {
                commands
                    .entity(entity)
                    .insert(OriginalColor(sprite.color));
            }
        }
    }

    // Fog of Warを適用
    for (_entity, tile_pos, mut sprite, original_color) in tile_query.iter_mut() {
        // 可視状態に応じて色を変更
        match exploration_data.map.get(tile_pos.x, tile_pos.y) {
            Some(TileVisibility::Visible) => {
                // 現在視界内: 元の色を使用
                if let Some(original) = original_color {
                    sprite.color = original.0;
                }
            }
            Some(TileVisibility::Explored) => {
                // 探索済み: 暗め
                sprite.color = Color::srgb(0.4, 0.4, 0.5);
            }
            Some(TileVisibility::Unexplored) | None => {
                // 未探索: 黒
                sprite.color = Color::BLACK;
            }
        }
    }
}

/// マップモード終了時にタイルの色を元に戻すシステム
pub fn restore_tile_colors_system(
    mut commands: Commands,
    map_mode_state: Res<MapModeState>,
    mut tile_query: Query<(Entity, &mut Sprite, Option<&OriginalColor>), With<MapTile>>,
) {
    // マップモードが無効になった瞬間のみ実行
    if !map_mode_state.enabled && map_mode_state.is_changed() {
        for (entity, mut sprite, original_color) in tile_query.iter_mut() {
            if let Some(original) = original_color {
                sprite.color = original.0;
                commands.entity(entity).remove::<OriginalColor>();
            }
        }
    }
}
