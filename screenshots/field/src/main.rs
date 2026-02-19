use bevy::prelude::*;

use app_state::PartyState;
use movement_ui::{ActiveMap, Player, TilePosition};
use screenshot_common::screenshot_app;
use terrain::Terrain;
use world_ui::{
    camera_follow, init_exploration_system, init_minimap_system, init_tile_pool, setup_camera,
    setup_hud, spawn_field_map, spawn_player, update_visible_tiles, MapModeState,
};

/// プレイヤーを海岸近くに移動するシステム（スクリーンショット用）
fn move_player_to_coast(
    active_map: Res<ActiveMap>,
    mut query: Query<(&mut TilePosition, &mut Transform), With<Player>>,
) {
    let Ok((mut tile_pos, mut transform)) = query.single_mut() else {
        return;
    };

    // 海に隣接する陸タイルを探す
    for y in 0..active_map.height {
        for x in 0..active_map.width {
            if active_map.grid[y][x] != Terrain::Plains {
                continue;
            }
            let has_sea_neighbor = [(-1i32, 0), (1, 0), (0, -1i32), (0, 1)]
                .iter()
                .any(|&(dx, dy)| {
                    let nx = (x as i32 + dx).rem_euclid(active_map.width as i32) as usize;
                    let ny = (y as i32 + dy).rem_euclid(active_map.height as i32) as usize;
                    active_map.grid[ny][nx] == Terrain::Sea
                });
            if has_sea_neighbor {
                tile_pos.x = x;
                tile_pos.y = y;
                let (wx, wy) = active_map.to_world(x, y);
                transform.translation.x = wx;
                transform.translation.y = wy;
                return;
            }
        }
    }
}

fn main() {
    let mut app = screenshot_app("field");
    app.init_resource::<MapModeState>()
        .init_resource::<PartyState>()
        .add_systems(
            Startup,
            (
                spawn_field_map,
                setup_camera,
                spawn_player,
                move_player_to_coast,
                init_tile_pool,
                init_exploration_system,
                init_minimap_system,
            )
                .chain(),
        )
        .add_systems(Startup, setup_hud)
        .add_systems(
            Update,
            (update_visible_tiles, camera_follow).chain(),
        )
        .run();
}
