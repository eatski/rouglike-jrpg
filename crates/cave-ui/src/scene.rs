use bevy::prelude::*;
use std::collections::HashMap;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use cave::{generate_cave_map, TreasureChest, TreasureContent, CAVE_HEIGHT, CAVE_WIDTH};
use party::ItemKind;
use terrain::Terrain;

use app_state::{Continent1CavePositions, OpenedChests};
use movement_ui::{
    Boat, Bounce, MapTile, MovementLocked, PendingMove, Player, SmoothMove, TilePosition,
};
use movement_ui::{ActiveMap, MovementState, WorldMapData, TILE_SIZE};

use world_ui::{spawn_boat_entities, BoatSpawnsResource, TileTextures};
use world_ui::{create_tile_pool, PooledTile, TilePool};

/// 洞窟進入前のフィールド座標を保存
#[derive(Resource)]
pub struct FieldReturnState {
    pub player_tile_x: usize,
    pub player_tile_y: usize,
}

/// 宝箱エンティティのマーカー
#[derive(Component)]
pub struct ChestEntity {
    pub treasure_index: usize,
}

/// 現在入っている洞窟の宝箱情報
#[derive(Resource)]
pub struct CaveTreasures {
    pub cave_pos: (usize, usize),
    pub treasures: Vec<TreasureChest>,
}

/// 洞窟内メッセージ表示の状態
#[derive(Resource, Default)]
pub struct CaveMessageState {
    pub message: Option<String>,
}

/// 洞窟メッセージ表示用UIマーカー
#[derive(Component)]
pub struct CaveMessageUI;

/// 洞窟用タイルエンティティのマーカー
#[derive(Component)]
pub struct CaveTile;

/// 洞窟タイルプールリソース
#[derive(Resource)]
pub struct CaveTilePool {
    pub active_tiles: HashMap<(i32, i32), Entity>,
    pub last_player_pos: Option<(i32, i32)>,
}

#[allow(clippy::too_many_arguments)]
pub fn setup_cave_scene(
    mut commands: Commands,
    mut player_query: Query<(&mut TilePosition, &mut Transform), With<Player>>,
    tile_pool_query: Query<Entity, With<PooledTile>>,
    boat_query: Query<(Entity, &TilePosition), (With<Boat>, Without<Player>)>,
    mut move_state: ResMut<MovementState>,
    active_map: Res<ActiveMap>,
    tile_textures: Res<TileTextures>,
    opened_chests: Res<OpenedChests>,
    mut boat_spawns: ResMut<BoatSpawnsResource>,
    continent1_caves: Res<Continent1CavePositions>,
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
    commands.insert_resource(WorldMapData(active_map.clone()));

    // 船の現在位置をBoatSpawnsResourceに同期（移動後の位置を保存）
    boat_spawns.positions = boat_query.iter().map(|(_, pos)| (pos.x, pos.y)).collect();

    // フィールドエンティティをdespawn
    for entity in &tile_pool_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TilePool>();

    for (entity, _) in &boat_query {
        commands.entity(entity).despawn();
    }

    // 洞窟マップ生成（ワールドマップ座標からシードを決定し、同じ洞窟は常に同じ形にする）
    let cave_world_pos = (tile_pos.x, tile_pos.y);
    let seed = tile_pos.x as u64 * 10007 + tile_pos.y as u64;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let guaranteed_items: Vec<TreasureContent> =
        if continent1_caves.positions.contains(&cave_world_pos) {
            vec![
                TreasureContent::Item(ItemKind::MoonFragment),
                TreasureContent::Item(ItemKind::MoonFragment),
            ]
        } else {
            vec![]
        };
    let cave_data = generate_cave_map(&mut rng, &guaranteed_items);
    let (spawn_x, spawn_y) = cave_data.spawn_position;

    // 洞窟用ActiveMapを作成
    let cave_origin_x = -(CAVE_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let cave_origin_y = -(CAVE_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;

    let active_map_resource = ActiveMap {
        grid: cave_data.grid,
        width: cave_data.width,
        height: cave_data.height,
        origin_x: cave_origin_x,
        origin_y: cave_origin_y,
    };

    // 宝箱エンティティをspawn（取得済みは開いた見た目で表示）
    let scale = TILE_SIZE / 16.0;
    let opened_set = opened_chests.chests.get(&cave_world_pos);
    for (i, treasure) in cave_data.treasures.iter().enumerate() {
        let is_opened = opened_set.is_some_and(|set| set.contains(&i));
        let texture = if is_opened {
            tile_textures.chest_open.clone()
        } else {
            tile_textures.chest.clone()
        };
        let (world_x, world_y) = active_map_resource.to_world_logical(treasure.x as i32, treasure.y as i32);
        commands.spawn((
            ChestEntity { treasure_index: i },
            Sprite::from_image(texture),
            Transform::from_xyz(world_x, world_y, 0.5).with_scale(Vec3::splat(scale)),
        ));
    }

    // 宝箱情報をリソースとして保存
    commands.insert_resource(CaveTreasures {
        cave_pos: cave_world_pos,
        treasures: cave_data.treasures,
    });
    commands.insert_resource(CaveMessageState::default());

    commands.insert_resource(active_map_resource);

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

pub fn despawn_cave_entities(
    mut commands: Commands,
    cave_tile_query: Query<Entity, With<CaveTile>>,
    chest_query: Query<Entity, With<ChestEntity>>,
    message_ui_query: Query<Entity, With<CaveMessageUI>>,
) {
    for entity in &cave_tile_query {
        commands.entity(entity).despawn();
    }
    for entity in &chest_query {
        commands.entity(entity).despawn();
    }
    for entity in &message_ui_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<CaveTilePool>();
    commands.remove_resource::<CaveTreasures>();
    commands.remove_resource::<CaveMessageState>();
}

pub fn restore_field_from_cave(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut TilePosition, &mut Transform), With<Player>>,
    field_return: Res<FieldReturnState>,
    tile_textures: Res<TileTextures>,
    boat_spawns: Res<BoatSpawnsResource>,
    mut move_state: ResMut<MovementState>,
    world_map: Res<WorldMapData>,
) {
    let restored_map = world_map.0.clone();

    if let Ok((entity, mut tile_pos, mut transform)) = player_query.single_mut() {
        tile_pos.x = field_return.player_tile_x;
        tile_pos.y = field_return.player_tile_y;

        let (world_x, world_y) = restored_map.to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;

        commands
            .entity(entity)
            .remove::<MovementLocked>()
            .remove::<SmoothMove>()
            .remove::<PendingMove>()
            .remove::<Bounce>();
    }

    commands.remove_resource::<FieldReturnState>();
    commands.remove_resource::<WorldMapData>();

    create_tile_pool(&mut commands, &tile_textures);
    spawn_boat_entities(&mut commands, &boat_spawns, &tile_textures, &restored_map);

    commands.insert_resource(restored_map);
    *move_state = MovementState::default();
}
