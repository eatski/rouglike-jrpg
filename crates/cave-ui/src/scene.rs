use bevy::prelude::*;
use std::collections::HashMap;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use cave::{generate_boss_cave_map, generate_cave_map, TreasureChest, TreasureContent, CAVE_HEIGHT, CAVE_WIDTH};
use party::ItemKind;

use app_state::{BossDefeated, ContinentCavePositions, ContinentMap, EncounterZone, OpenedChests};
use field_core::{ActiveMap, Boat, Player, TilePosition, WorldMapData, TILE_SIZE};
use field_walk_ui::MovementState;

use field_walk_ui::{spawn_boat_entities, BoatSpawnsResource, BossCaveWorldPos, MapModeState, TileTextures};
use field_walk_ui::{create_tile_pool, PooledTile, SimpleTile, SimpleTileMap, TilePool};

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

/// ボスエンティティのマーカー
#[derive(Component)]
pub struct BossEntity {
    pub tile_x: usize,
    pub tile_y: usize,
}

/// ボス洞窟の状態リソース
#[derive(Resource)]
pub struct BossCaveState {
    pub boss_position: (usize, usize),
    pub cave_world_pos: (usize, usize),
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
    mut map_mode_state: ResMut<MapModeState>,
    continent_caves: Res<ContinentCavePositions>,
    continent_map: Option<Res<ContinentMap>>,
) {
    // ワールドでマップモードがONのまま洞窟に入った場合にリセット
    map_mode_state.enabled = false;
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

    // 洞窟のワールド座標を保存し、エンカウントゾーンを設定
    let cave_world_pos = (tile_pos.x, tile_pos.y);
    let cave_continent_id = continent_map
        .as_ref()
        .and_then(|cm| cm.get(cave_world_pos.0, cave_world_pos.1))
        .unwrap_or(0);
    commands.insert_resource(EncounterZone {
        continent_id: cave_continent_id,
        is_cave: true,
    });

    // 洞窟マップ生成（ワールドマップ座標からシードを決定し、同じ洞窟は常に同じ形にする）
    let seed = tile_pos.x as u64 * 10007 + tile_pos.y as u64;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let guaranteed_items: Vec<TreasureContent> =
        if continent_caves
            .caves_by_continent
            .iter()
            .any(|caves| caves.contains(&cave_world_pos))
        {
            vec![
                TreasureContent::Item(ItemKind::MoonFragment),
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
        wraps: false,
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

    // プレイヤーを洞窟のスポーン位置に移動（insert_resourceでmoveされる前に座標計算）
    tile_pos.x = spawn_x;
    tile_pos.y = spawn_y;
    let (world_x, world_y) = active_map_resource.to_world(spawn_x, spawn_y);
    transform.translation.x = world_x;
    transform.translation.y = world_y;

    commands.insert_resource(active_map_resource);

    // 洞窟用タイルプールを初期化
    commands.insert_resource(SimpleTileMap {
        active_tiles: HashMap::new(),
        last_player_pos: None,
    });

    // MovementStateをリセット
    *move_state = MovementState::default();
}

/// ボス洞窟シーンのセットアップ
#[allow(clippy::too_many_arguments)]
pub fn setup_boss_cave_scene(
    mut commands: Commands,
    mut player_query: Query<(&mut TilePosition, &mut Transform), With<Player>>,
    tile_pool_query: Query<Entity, With<PooledTile>>,
    boat_query: Query<(Entity, &TilePosition), (With<Boat>, Without<Player>)>,
    mut move_state: ResMut<MovementState>,
    active_map: Res<ActiveMap>,
    tile_textures: Res<TileTextures>,
    _boss_cave_world_pos: Res<BossCaveWorldPos>,
    boss_defeated: Option<Res<BossDefeated>>,
    mut boat_spawns: ResMut<BoatSpawnsResource>,
) {
    let Ok((mut tile_pos, mut transform)) = player_query.single_mut() else {
        return;
    };

    // フィールド座標を保存
    commands.insert_resource(FieldReturnState {
        player_tile_x: tile_pos.x,
        player_tile_y: tile_pos.y,
    });

    // ワールドマップを退避
    commands.insert_resource(WorldMapData(active_map.clone()));

    // 船の現在位置をBoatSpawnsResourceに同期
    boat_spawns.positions = boat_query.iter().map(|(_, pos)| (pos.x, pos.y)).collect();

    // フィールドエンティティをdespawn
    for entity in &tile_pool_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TilePool>();

    for (entity, _) in &boat_query {
        commands.entity(entity).despawn();
    }

    // ボス洞窟マップ生成
    let cave_world_pos = (tile_pos.x, tile_pos.y);
    let seed = tile_pos.x as u64 * 10007 + tile_pos.y as u64 + 999;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let cave_data = generate_boss_cave_map(&mut rng);
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
        wraps: false,
    };

    // ボス洞窟状態リソース
    commands.insert_resource(BossCaveState {
        boss_position: cave_data.boss_position,
        cave_world_pos,
    });

    // ボススプライトを配置（撃破済みでなければ）
    if boss_defeated.is_none() {
        let scale = TILE_SIZE / 16.0;
        let (boss_world_x, boss_world_y) = active_map_resource.to_world_logical(
            cave_data.boss_position.0 as i32,
            cave_data.boss_position.1 as i32,
        );
        commands.spawn((
            BossEntity {
                tile_x: cave_data.boss_position.0,
                tile_y: cave_data.boss_position.1,
            },
            Sprite::from_image(tile_textures.dark_lord.clone()),
            Transform::from_xyz(boss_world_x, boss_world_y, 0.5).with_scale(Vec3::splat(scale)),
        ));
    }

    commands.insert_resource(CaveMessageState::default());
    commands.insert_resource(active_map_resource);

    // プレイヤーをスポーン位置に移動
    tile_pos.x = spawn_x;
    tile_pos.y = spawn_y;
    let world_x = cave_origin_x + spawn_x as f32 * TILE_SIZE;
    let world_y = cave_origin_y + spawn_y as f32 * TILE_SIZE;
    transform.translation.x = world_x;
    transform.translation.y = world_y;

    // 洞窟用タイルプールを初期化
    commands.insert_resource(SimpleTileMap {
        active_tiles: HashMap::new(),
        last_player_pos: None,
    });

    *move_state = MovementState::default();
}

pub fn despawn_cave_entities(
    mut commands: Commands,
    cave_tile_query: Query<Entity, With<SimpleTile>>,
    chest_query: Query<Entity, With<ChestEntity>>,
    message_ui_query: Query<Entity, With<CaveMessageUI>>,
    boss_query: Query<Entity, With<BossEntity>>,
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
    for entity in &boss_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<SimpleTileMap>();
    commands.remove_resource::<CaveTreasures>();
    commands.remove_resource::<CaveMessageState>();
    commands.remove_resource::<BossCaveState>();
}

pub fn restore_field_from_cave(
    mut commands: Commands,
    mut player_query: Query<(&mut TilePosition, &mut Transform), With<Player>>,
    field_return: Res<FieldReturnState>,
    tile_textures: Res<TileTextures>,
    boat_spawns: Res<BoatSpawnsResource>,
    world_map: Res<WorldMapData>,
) {
    let restored_map = world_map.0.clone();

    if let Ok((mut tile_pos, mut transform)) = player_query.single_mut() {
        tile_pos.x = field_return.player_tile_x;
        tile_pos.y = field_return.player_tile_y;
        let (world_x, world_y) =
            restored_map.to_world(field_return.player_tile_x, field_return.player_tile_y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }

    commands.remove_resource::<FieldReturnState>();
    commands.remove_resource::<WorldMapData>();

    create_tile_pool(&mut commands, &tile_textures);
    spawn_boat_entities(&mut commands, &boat_spawns, &tile_textures, &restored_map);

    commands.insert_resource(restored_map);
}
