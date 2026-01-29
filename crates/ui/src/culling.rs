use bevy::prelude::*;

use crate::components::MapTile;
use crate::constants::{CULLING_MARGIN, VISIBLE_SIZE};
use crate::map_mode::MapModeState;

pub fn tile_culling(
    map_mode_state: Res<MapModeState>,
    camera_query: Query<&Transform, (With<Camera2d>, Changed<Transform>)>,
    mut tile_query: Query<(&Transform, &mut Visibility), (With<MapTile>, Without<Camera2d>)>,
) {
    // マップモード時はカリングをスキップ（ミニマップで表示するため）
    if map_mode_state.enabled {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_x = camera_transform.translation.x;
    let camera_y = camera_transform.translation.y;

    let half_visible = VISIBLE_SIZE / 2.0;
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
