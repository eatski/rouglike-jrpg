use std::collections::HashMap;

use bevy::prelude::*;
use terrain::Terrain;

use field_core::{ActiveMap, MapTile, Player, TilePosition, TILE_SIZE};
use crate::SmoothMove;
use crate::smooth_move::is_smooth_moving;

use crate::{MapModeState, TileTextures};

/// シンプルタイル（despawn/spawn方式）のマーカーコンポーネント
#[derive(Component)]
pub struct SimpleTile;

/// シンプルタイルマップのリソース
#[derive(Resource)]
pub struct SimpleTileMap {
    pub active_tiles: HashMap<(i32, i32), Entity>,
    pub last_player_pos: Option<(i32, i32)>,
}

/// シンプルタイルの表示を更新するシステム
#[allow(clippy::too_many_arguments)]
pub fn update_simple_tiles(
    mut commands: Commands,
    active_map: Res<ActiveMap>,
    tile_textures: Res<TileTextures>,
    player_query: Query<&TilePosition, With<Player>>,
    mut tile_map: ResMut<SimpleTileMap>,
    smooth_move_query: Query<&SmoothMove, With<Player>>,
    mut tile_query: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<SimpleTile>>,
    map_mode_state: Res<MapModeState>,
) {
    // SmoothMove中はスキップ（完了フレーム以外）
    if is_smooth_moving(&smooth_move_query) {
        return;
    }

    let Ok(player_pos) = player_query.single() else {
        return;
    };

    let player_tile = (player_pos.x as i32, player_pos.y as i32);

    // マップモード切替時は強制再描画
    let force_redraw = map_mode_state.is_changed();

    if tile_map.last_player_pos == Some(player_tile) && !force_redraw {
        return;
    }
    tile_map.last_player_pos = Some(player_tile);

    let half = 7i32; // 表示範囲（片側）
    let scale = TILE_SIZE / 16.0;

    // 新しい表示範囲
    let mut needed: Vec<(i32, i32)> = Vec::new();
    if map_mode_state.enabled {
        // マップモード: 全タイルを描画
        for ly in 0..active_map.height as i32 {
            for lx in 0..active_map.width as i32 {
                needed.push((lx, ly));
            }
        }
    } else {
        for dy in -half..=half {
            for dx in -half..=half {
                let lx = player_tile.0 + dx;
                let ly = player_tile.1 + dy;
                needed.push((lx, ly));
            }
        }
    }

    // 範囲外のタイルを削除
    let to_remove: Vec<(i32, i32)> = tile_map
        .active_tiles
        .keys()
        .filter(|pos| !needed.contains(pos))
        .copied()
        .collect();

    for pos in to_remove {
        if let Some(entity) = tile_map.active_tiles.remove(&pos) {
            if let Ok((_, _, mut vis)) = tile_query.get_mut(entity) {
                *vis = Visibility::Hidden;
            }
            commands.entity(entity).despawn();
        }
    }

    // 新しいタイルを生成
    for (lx, ly) in needed {
        if tile_map.active_tiles.contains_key(&(lx, ly)) {
            continue;
        }

        // 範囲外は壁として描画
        let terrain = if lx >= 0
            && lx < active_map.width as i32
            && ly >= 0
            && ly < active_map.height as i32
        {
            active_map.grid[ly as usize][lx as usize]
        } else {
            Terrain::CaveWall
        };

        let texture = match terrain {
            Terrain::CaveWall => tile_textures.cave_wall.clone(),
            Terrain::CaveFloor => tile_textures.cave_floor.clone(),
            Terrain::BossCaveWall => tile_textures.boss_cave_wall.clone(),
            Terrain::BossCaveFloor => tile_textures.boss_cave_floor.clone(),
            Terrain::WarpZone => tile_textures.warp_zone.clone(),
            Terrain::Ladder => tile_textures.ladder.clone(),
            _ => tile_textures.cave_wall.clone(),
        };

        let (world_x, world_y) = active_map.to_world_logical(lx, ly);

        let entity = commands
            .spawn((
                MapTile,
                SimpleTile,
                Sprite::from_image(texture),
                Transform::from_xyz(world_x, world_y, 0.0).with_scale(Vec3::splat(scale)),
            ))
            .id();

        tile_map.active_tiles.insert((lx, ly), entity);
    }
}
