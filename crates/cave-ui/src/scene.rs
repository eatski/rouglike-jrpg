use bevy::prelude::*;
use std::collections::HashMap;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use cave::{generate_cave_map, CAVE_HEIGHT, CAVE_WIDTH};
use terrain::Terrain;

use movement_ui::{
    Boat, Bounce, MapTile, MovementLocked, PendingMove, Player, SmoothMove, TilePosition,
};
use shared_ui::{ActiveMap, MovementState, WorldMapData, TILE_SIZE};

use world_ui::{spawn_boat_entities, BoatSpawnsResource, TileTextures};
use world_ui::{create_tile_pool, PooledTile, TilePool};

/// 洞窟進入前のフィールド座標を保存
#[derive(Resource)]
pub struct FieldReturnState {
    pub player_tile_x: usize,
    pub player_tile_y: usize,
}

/// 洞窟用タイルエンティティのマーカー
#[derive(Component)]
pub struct CaveTile;

/// 洞窟タイルプールリソース
#[derive(Resource)]
pub struct CaveTilePool {
    pub active_tiles: HashMap<(i32, i32), Entity>,
    pub last_player_pos: Option<(i32, i32)>,
}

pub fn setup_cave_scene(
    mut commands: Commands,
    mut player_query: Query<(&mut TilePosition, &mut Transform), With<Player>>,
    tile_pool_query: Query<Entity, With<PooledTile>>,
    boat_query: Query<Entity, With<Boat>>,
    mut move_state: ResMut<MovementState>,
    active_map: Res<ActiveMap>,
) {
    // フィールド座標を保存
    let Ok((mut tile_pos, mut transform)) = player_query.single_mut() else {
        return;
    };

    commands.insert_resource(FieldReturnState {
        player_tile_x: tile_pos.x,
        player_tile_y: tile_pos.y,
    });

    // ワールドマップを退避
    commands.insert_resource(WorldMapData {
        grid: active_map.grid.clone(),
        width: active_map.width,
        height: active_map.height,
        origin_x: active_map.origin_x,
        origin_y: active_map.origin_y,
    });

    // フィールドエンティティをdespawn
    for entity in &tile_pool_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TilePool>();

    for entity in &boat_query {
        commands.entity(entity).despawn();
    }

    // 洞窟マップ生成（ワールドマップ座標からシードを決定し、同じ洞窟は常に同じ形にする）
    let seed = tile_pos.x as u64 * 10007 + tile_pos.y as u64;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let cave_data = generate_cave_map(&mut rng);
    let (spawn_x, spawn_y) = cave_data.spawn_position;

    // 洞窟用ActiveMapを作成
    let cave_origin_x = -(CAVE_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let cave_origin_y = -(CAVE_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    commands.insert_resource(ActiveMap {
        grid: cave_data.grid,
        width: cave_data.width,
        height: cave_data.height,
        origin_x: cave_origin_x,
        origin_y: cave_origin_y,
    });

    // プレイヤーを洞窟のスポーン位置に移動
    tile_pos.x = spawn_x;
    tile_pos.y = spawn_y;
    let world_x = cave_origin_x + spawn_x as f32 * TILE_SIZE;
    let world_y = cave_origin_y + spawn_y as f32 * TILE_SIZE;
    transform.translation.x = world_x;
    transform.translation.y = world_y;

    // 洞窟用タイルプールを初期化
    commands.insert_resource(CaveTilePool {
        active_tiles: HashMap::new(),
        last_player_pos: None,
    });

    // MovementStateをリセット
    *move_state = MovementState::default();
}

/// 洞窟タイルの表示を更新するシステム
pub fn update_cave_tiles(
    mut commands: Commands,
    active_map: Res<ActiveMap>,
    tile_textures: Res<TileTextures>,
    player_query: Query<&TilePosition, With<Player>>,
    mut cave_pool: ResMut<CaveTilePool>,
    smooth_move_query: Query<&SmoothMove, With<Player>>,
    mut tile_query: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<CaveTile>>,
) {
    // SmoothMove中はスキップ（完了フレーム以外）
    for smooth_move in smooth_move_query.iter() {
        if !smooth_move.timer.just_finished() {
            return;
        }
    }

    let Ok(player_pos) = player_query.single() else {
        return;
    };

    let player_tile = (player_pos.x as i32, player_pos.y as i32);

    if cave_pool.last_player_pos == Some(player_tile) {
        return;
    }
    cave_pool.last_player_pos = Some(player_tile);

    let half = 7i32; // 表示範囲（片側）
    let scale = TILE_SIZE / 16.0;

    // 新しい表示範囲
    let mut needed: Vec<(i32, i32)> = Vec::new();
    for dy in -half..=half {
        for dx in -half..=half {
            let lx = player_tile.0 + dx;
            let ly = player_tile.1 + dy;
            needed.push((lx, ly));
        }
    }

    // 範囲外のタイルを削除
    let to_remove: Vec<(i32, i32)> = cave_pool
        .active_tiles
        .keys()
        .filter(|pos| !needed.contains(pos))
        .copied()
        .collect();

    for pos in to_remove {
        if let Some(entity) = cave_pool.active_tiles.remove(&pos) {
            if let Ok((_, _, mut vis)) = tile_query.get_mut(entity) {
                *vis = Visibility::Hidden;
            }
            commands.entity(entity).despawn();
        }
    }

    // 新しいタイルを生成
    for (lx, ly) in needed {
        if cave_pool.active_tiles.contains_key(&(lx, ly)) {
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
            Terrain::WarpZone => tile_textures.warp_zone.clone(),
            Terrain::Ladder => tile_textures.ladder.clone(),
            _ => tile_textures.cave_wall.clone(),
        };

        let (world_x, world_y) = active_map.to_world_logical(lx, ly);

        let entity = commands
            .spawn((
                MapTile,
                CaveTile,
                Sprite::from_image(texture),
                Transform::from_xyz(world_x, world_y, 0.0).with_scale(Vec3::splat(scale)),
            ))
            .id();

        cave_pool.active_tiles.insert((lx, ly), entity);
    }
}

pub fn cleanup_cave_scene(
    mut commands: Commands,
    cave_tile_query: Query<Entity, With<CaveTile>>,
    mut player_query: Query<(Entity, &mut TilePosition, &mut Transform), With<Player>>,
    field_return: Res<FieldReturnState>,
    tile_textures: Res<TileTextures>,
    boat_spawns: Res<BoatSpawnsResource>,
    mut move_state: ResMut<MovementState>,
    world_map: Res<WorldMapData>,
) {
    // 洞窟タイルを全て削除
    for entity in &cave_tile_query {
        commands.entity(entity).despawn();
    }

    // 洞窟リソースを削除
    commands.remove_resource::<CaveTilePool>();

    // WorldMapDataからActiveMapを復元
    let restored_map = ActiveMap {
        grid: world_map.grid.clone(),
        width: world_map.width,
        height: world_map.height,
        origin_x: world_map.origin_x,
        origin_y: world_map.origin_y,
    };

    // プレイヤーをフィールド座標に復元
    if let Ok((entity, mut tile_pos, mut transform)) = player_query.single_mut() {
        tile_pos.x = field_return.player_tile_x;
        tile_pos.y = field_return.player_tile_y;

        let (world_x, world_y) = restored_map.to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;

        // 移動関連コンポーネントをクリーンアップ
        commands
            .entity(entity)
            .remove::<MovementLocked>()
            .remove::<SmoothMove>()
            .remove::<PendingMove>()
            .remove::<Bounce>();
    }

    commands.remove_resource::<FieldReturnState>();
    commands.remove_resource::<WorldMapData>();

    // フィールドエンティティをrespawn
    create_tile_pool(&mut commands, &tile_textures);
    spawn_boat_entities(&mut commands, &boat_spawns, &tile_textures, &restored_map);

    commands.insert_resource(restored_map);
    *move_state = MovementState::default();
}
