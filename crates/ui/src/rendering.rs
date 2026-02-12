use bevy::prelude::*;

use game::map::{calculate_boat_spawns, generate_connected_map};

use crate::components::{Boat, Player, TilePosition};
use crate::resources::{MapDataResource, SpawnPosition};

use super::constants::{tile_to_world, TILE_SIZE};

#[derive(Resource)]
pub struct TileTextures {
    pub sea: Handle<Image>,
    pub plains: Handle<Image>,
    pub forest: Handle<Image>,
    pub mountain: Handle<Image>,
    pub boat: Handle<Image>,
    pub town: Handle<Image>,
    pub cave: Handle<Image>,
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
    };

    let mut rng = rand::thread_rng();
    let map_data = generate_connected_map(&mut rng);

    // 船のスポーン位置を計算
    let boat_spawns = calculate_boat_spawns(&map_data.grid, &mut rng);

    commands.insert_resource(SpawnPosition {
        x: map_data.spawn_position.0,
        y: map_data.spawn_position.1,
    });

    // 船をスポーン
    let scale = TILE_SIZE / 16.0;

    for spawn in &boat_spawns {
        let (world_x, world_y) = tile_to_world(spawn.x, spawn.y);

        commands.spawn((
            Boat,
            TilePosition {
                x: spawn.x,
                y: spawn.y,
            },
            Sprite::from_image(tile_textures.boat.clone()),
            Transform::from_xyz(world_x, world_y, 0.5).with_scale(Vec3::splat(scale)),
        ));
    }

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

