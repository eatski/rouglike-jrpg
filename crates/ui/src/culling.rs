use bevy::prelude::*;

use crate::components::MapTile;
use crate::constants::{CULLING_MARGIN, VISIBLE_SIZE};
use crate::map_mode::{MapModeState, MAP_MODE_ZOOM};

pub fn tile_culling(
    map_mode_state: Res<MapModeState>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut tile_query: Query<(&Transform, &mut Visibility), (With<MapTile>, Without<Camera2d>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_x = camera_transform.translation.x;
    let camera_y = camera_transform.translation.y;

    // マップモード時は広い範囲を表示
    let visible_size = if map_mode_state.enabled {
        MAP_MODE_ZOOM
    } else {
        VISIBLE_SIZE
    };

    let half_visible = visible_size / 2.0;
    let left = camera_x - half_visible - CULLING_MARGIN;
    let right = camera_x + half_visible + CULLING_MARGIN;
    let bottom = camera_y - half_visible - CULLING_MARGIN;
    let top = camera_y + half_visible + CULLING_MARGIN;

    for (tile_transform, mut visibility) in tile_query.iter_mut() {
        let tile_x = tile_transform.translation.x;
        let tile_y = tile_transform.translation.y;

        let is_visible = tile_x >= left && tile_x <= right
                      && tile_y >= bottom && tile_y <= top;

        let new_visibility = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if *visibility != new_visibility {
            *visibility = new_visibility;
        }
    }
}
