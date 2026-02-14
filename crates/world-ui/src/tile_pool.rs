use bevy::prelude::*;
use std::collections::HashMap;

use terrain::{Terrain, MAP_HEIGHT, MAP_WIDTH};

use movement_ui::{MapTile, Player, SmoothMove, TilePosition};
use shared_ui::{logical_to_world, MapDataResource, TILE_SIZE, VISIBLE_CELLS};

use crate::rendering::TileTextures;

/// 視界範囲 + バッファのサイズ（片側）
const TILE_BUFFER: i32 = 3;
/// プール内のタイル幅
const TILE_POOL_WIDTH: i32 = VISIBLE_CELLS as i32 + TILE_BUFFER * 2; // 15
/// プール内のタイル総数
const TILE_POOL_SIZE: usize = (TILE_POOL_WIDTH * TILE_POOL_WIDTH) as usize; // 225

/// タイルプールを管理するリソース
#[derive(Resource)]
pub struct TilePool {
    /// 再利用可能なタイルエンティティ
    available: Vec<Entity>,
    /// 現在表示中のタイル: (論理座標) -> Entity
    active_tiles: HashMap<(i32, i32), Entity>,
    /// 前回のプレイヤータイル位置
    pub last_player_pos: Option<(i32, i32)>,
}

impl TilePool {
    fn new() -> Self {
        Self {
            available: Vec::with_capacity(TILE_POOL_SIZE),
            active_tiles: HashMap::with_capacity(TILE_POOL_SIZE),
            last_player_pos: None,
        }
    }
}

/// プールされたタイルの論理座標を保持するコンポーネント
#[derive(Component)]
pub struct PooledTile {
    pub logical_x: i32,
    pub logical_y: i32,
}

/// タイルプールを生成してリソースとして登録する
pub fn create_tile_pool(commands: &mut Commands, tile_textures: &TileTextures) {
    let scale = TILE_SIZE / 16.0;
    let mut pool = TilePool::new();

    for _ in 0..TILE_POOL_SIZE {
        let entity = commands
            .spawn((
                MapTile,
                PooledTile {
                    logical_x: 0,
                    logical_y: 0,
                },
                Sprite::from_image(tile_textures.sea.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(scale)),
                Visibility::Hidden,
            ))
            .id();

        pool.available.push(entity);
    }

    commands.insert_resource(pool);
}

/// タイルプールを初期化するシステム（Startup用）
pub fn init_tile_pool(mut commands: Commands, tile_textures: Res<TileTextures>) {
    create_tile_pool(&mut commands, &tile_textures);
}

/// 論理座標をマップ座標に変換（トーラスラップ）
fn logical_to_map(logical_x: i32, logical_y: i32) -> (usize, usize) {
    (
        logical_x.rem_euclid(MAP_WIDTH as i32) as usize,
        logical_y.rem_euclid(MAP_HEIGHT as i32) as usize,
    )
}

/// 地形に対応するテクスチャを取得
fn get_terrain_texture(terrain: Terrain, textures: &TileTextures) -> Handle<Image> {
    match terrain {
        Terrain::Sea => textures.sea.clone(),
        Terrain::Plains => textures.plains.clone(),
        Terrain::Forest => textures.forest.clone(),
        Terrain::Mountain => textures.mountain.clone(),
        Terrain::Town => textures.town.clone(),
        Terrain::Cave => textures.cave.clone(),
    }
}

/// プレイヤー位置に応じて可視タイルを更新するシステム
pub fn update_visible_tiles(
    mut tile_pool: ResMut<TilePool>,
    player_query: Query<&TilePosition, With<Player>>,
    smooth_move_query: Query<&SmoothMove, With<Player>>,
    map_data: Res<MapDataResource>,
    tile_textures: Res<TileTextures>,
    mut tile_query: Query<(
        &mut Transform,
        &mut Sprite,
        &mut PooledTile,
        &mut Visibility,
    )>,
) {
    // スムーズ移動のアニメーション中はタイル更新をスキップ
    // ただし完了フレームは更新する（マップ端ラップ時の暗転防止）
    for smooth_move in smooth_move_query.iter() {
        if !smooth_move.timer.just_finished() {
            return;
        }
    }

    let Ok(player_pos) = player_query.single() else {
        return;
    };

    let player_tile = (player_pos.x as i32, player_pos.y as i32);

    // プレイヤーが移動していなければスキップ
    if tile_pool.last_player_pos == Some(player_tile) {
        return;
    }
    tile_pool.last_player_pos = Some(player_tile);

    let half_width = TILE_POOL_WIDTH / 2;

    // 新しい表示範囲を計算
    let mut needed_tiles: Vec<(i32, i32)> = Vec::with_capacity(TILE_POOL_SIZE);
    for dy in -half_width..=half_width {
        for dx in -half_width..=half_width {
            let logical_x = player_tile.0 + dx;
            let logical_y = player_tile.1 + dy;
            needed_tiles.push((logical_x, logical_y));
        }
    }

    // 不要になったタイルをプールに戻す
    let tiles_to_recycle: Vec<(i32, i32)> = tile_pool
        .active_tiles
        .keys()
        .filter(|pos| !needed_tiles.contains(pos))
        .copied()
        .collect();

    for pos in tiles_to_recycle {
        if let Some(entity) = tile_pool.active_tiles.remove(&pos) {
            // 非表示にしてプールに戻す
            if let Ok((_, _, _, mut visibility)) = tile_query.get_mut(entity) {
                *visibility = Visibility::Hidden;
            }
            tile_pool.available.push(entity);
        }
    }

    // 新しく必要なタイルを配置
    for (logical_x, logical_y) in needed_tiles {
        if tile_pool.active_tiles.contains_key(&(logical_x, logical_y)) {
            continue; // 既に表示中
        }

        let Some(entity) = tile_pool.available.pop() else {
            continue; // プールが空（通常は発生しない）
        };

        // マップ座標を計算（トーラスラップ）
        let (map_x, map_y) = logical_to_map(logical_x, logical_y);
        let terrain = map_data.grid[map_y][map_x];
        let texture = get_terrain_texture(terrain, &tile_textures);

        // ワールド座標を計算
        let (world_x, world_y) = logical_to_world(logical_x, logical_y);

        if let Ok((mut transform, mut sprite, mut pooled, mut visibility)) =
            tile_query.get_mut(entity)
        {
            transform.translation.x = world_x;
            transform.translation.y = world_y;
            sprite.image = texture;
            pooled.logical_x = logical_x;
            pooled.logical_y = logical_y;
            *visibility = Visibility::Visible;
        }

        tile_pool.active_tiles.insert((logical_x, logical_y), entity);
    }
}
