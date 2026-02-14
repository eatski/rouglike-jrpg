use bevy::prelude::*;

use world::map::{calculate_boat_spawns, generate_connected_map};

use components_ui::{Boat, Player, TilePosition};
use shared_ui::{tile_to_world, MapDataResource, TILE_SIZE};

use crate::resources::SpawnPosition;

/// 船のスポーン位置を保存するリソース（洞窟脱出後の復元に使用）
#[derive(Resource)]
pub struct BoatSpawnsResource {
    pub positions: Vec<(usize, usize)>,
}

/// 保存された位置に船エンティティをスポーンする
pub fn spawn_boat_entities(
    commands: &mut Commands,
    boat_spawns: &BoatSpawnsResource,
    tile_textures: &TileTextures,
) {
    let scale = TILE_SIZE / 16.0;
    for &(x, y) in &boat_spawns.positions {
        let (world_x, world_y) = tile_to_world(x, y);
        commands.spawn((
            Boat,
            TilePosition { x, y },
            Sprite::from_image(tile_textures.boat.clone()),
            Transform::from_xyz(world_x, world_y, 0.5).with_scale(Vec3::splat(scale)),
        ));
    }
}

#[derive(Resource)]
pub struct TileTextures {
    pub sea: Handle<Image>,
    pub plains: Handle<Image>,
    pub forest: Handle<Image>,
    pub mountain: Handle<Image>,
    pub boat: Handle<Image>,
    pub town: Handle<Image>,
    pub cave: Handle<Image>,
    pub cave_wall: Handle<Image>,
    pub cave_floor: Handle<Image>,
    pub warp_zone: Handle<Image>,
}

pub fn spawn_field_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    // テクスチャをロード
    let tile_textures = TileTextures {
        sea: asset_server.load("tiles/sea.png"),
        plains: asset_server.load("tiles/plains.png"),
        forest: asset_server.load("tiles/forest.png"),
        mountain: asset_server.load("tiles/mountain.png"),
        boat: asset_server.load("tiles/boat.png"),
        town: asset_server.load("tiles/town.png"),
        cave: asset_server.load("tiles/cave.png"),
        cave_wall: asset_server.load("tiles/cave_wall.png"),
        cave_floor: asset_server.load("tiles/cave_floor.png"),
        warp_zone: asset_server.load("tiles/warp_zone.png"),
    };

    let mut rng = rand::thread_rng();
    let map_data = generate_connected_map(&mut rng);

    // 船のスポーン位置を計算
    let boat_spawns = calculate_boat_spawns(&map_data.grid, &mut rng);

    commands.insert_resource(SpawnPosition {
        x: map_data.spawn_position.0,
        y: map_data.spawn_position.1,
    });

    // 船のスポーン位置を保存してスポーン
    let boat_spawns_resource = BoatSpawnsResource {
        positions: boat_spawns.iter().map(|s| (s.x, s.y)).collect(),
    };
    spawn_boat_entities(&mut commands, &boat_spawns_resource, &tile_textures);
    commands.insert_resource(boat_spawns_resource);

    // タイルテクスチャをリソースとして登録（タイルプールで使用）
    commands.insert_resource(tile_textures);
    commands.insert_resource(MapDataResource::from(map_data));
}

pub fn spawn_player(
    mut commands: Commands,
    spawn_pos: Res<SpawnPosition>,
    asset_server: Res<AssetServer>,
) {
    let (world_x, world_y) = tile_to_world(spawn_pos.x, spawn_pos.y);

    let player_texture: Handle<Image> = asset_server.load("characters/player.png");
    let scale = TILE_SIZE / 16.0;

    commands.spawn((
        Player,
        TilePosition {
            x: spawn_pos.x,
            y: spawn_pos.y,
        },
        Sprite::from_image(player_texture),
        Transform::from_xyz(world_x, world_y, 1.0).with_scale(Vec3::splat(scale)),
    ));
}
