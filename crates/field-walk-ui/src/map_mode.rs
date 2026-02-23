use bevy::camera::Projection;
use bevy::prelude::*;

use movement_ui::{ActiveMap, TILE_SIZE, VISIBLE_SIZE};

/// 通常時のズーム値
pub const NORMAL_ZOOM: f32 = VISIBLE_SIZE;

/// マップモードの状態を管理するリソース
#[derive(Resource, Default)]
pub struct MapModeState {
    pub enabled: bool,
}

/// Mキー押下でマップモードをトグルするシステム
pub fn toggle_map_mode_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut map_mode_state: ResMut<MapModeState>,
    mut camera_query: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
    active_map: Res<ActiveMap>,
) {
    if input_ui::is_map_toggle_just_pressed(&keyboard) {
        map_mode_state.enabled = !map_mode_state.enabled;

        let map_mode_zoom = active_map.width as f32 * TILE_SIZE;

        // カメラズームと位置を変更
        if let Ok((mut projection, mut transform)) = camera_query.single_mut()
            && let Projection::Orthographic(ref mut ortho) = *projection
        {
            if map_mode_state.enabled {
                // マップモード: ズームアウト＆ワールド中心に固定
                ortho.scaling_mode = bevy::camera::ScalingMode::Fixed {
                    width: map_mode_zoom,
                    height: map_mode_zoom,
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

/// シーン遷移時にマップモードをリセットするシステム
pub fn reset_map_mode_system(
    mut map_mode_state: ResMut<MapModeState>,
    mut camera_query: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
) {
    map_mode_state.enabled = false;

    if let Ok((mut projection, _)) = camera_query.single_mut()
        && let Projection::Orthographic(ref mut ortho) = *projection
    {
        ortho.scaling_mode = bevy::camera::ScalingMode::Fixed {
            width: NORMAL_ZOOM,
            height: NORMAL_ZOOM,
        };
    }
}
