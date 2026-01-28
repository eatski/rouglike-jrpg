use bevy::prelude::*;

use game::map::{generate_map, Terrain, MAP_HEIGHT, MAP_WIDTH};

use crate::components::{MapTile, Player, TilePosition};
use crate::resources::{MapDataResource, SpawnPosition};

use super::constants::{MAP_PIXEL_HEIGHT, MAP_PIXEL_WIDTH, TILE_SIZE};

#[derive(Resource)]
pub struct TileTextures {
    pub sea: Handle<Image>,
    pub plains: Handle<Image>,
    pub forest: Handle<Image>,
    pub mountain: Handle<Image>,
}

pub fn spawn_field_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    // テクスチャをロード
    let tile_textures = TileTextures {
        sea: asset_server.load("tiles/sea.png"),
        plains: asset_server.load("tiles/plains.png"),
        forest: asset_server.load("tiles/forest.png"),
        mountain: asset_server.load("tiles/mountain.png"),
    };

    let origin_x = -MAP_PIXEL_WIDTH / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -MAP_PIXEL_HEIGHT / 2.0 + TILE_SIZE / 2.0;
    let mut rng = rand::thread_rng();
    let map_data = generate_map(&mut rng);

    commands.insert_resource(SpawnPosition {
        x: map_data.spawn_position.0,
        y: map_data.spawn_position.1,
    });

    // スプライトのスケール（16pxのテクスチャをTILE_SIZEに合わせる）
    let scale = TILE_SIZE / 16.0;

    // マップを3x3で複製描画（トーラスラップの視覚化）
    for offset_y in -1..=1 {
        for offset_x in -1..=1 {
            let base_x = origin_x + offset_x as f32 * MAP_PIXEL_WIDTH;
            let base_y = origin_y + offset_y as f32 * MAP_PIXEL_HEIGHT;

            for y in 0..MAP_HEIGHT {
                for x in 0..MAP_WIDTH {
                    let terrain = map_data.grid[y][x];
                    let world_x = base_x + x as f32 * TILE_SIZE;
                    let world_y = base_y + y as f32 * TILE_SIZE;

                    let texture = match terrain {
                        Terrain::Sea => tile_textures.sea.clone(),
                        Terrain::Plains => tile_textures.plains.clone(),
                        Terrain::Forest => tile_textures.forest.clone(),
                        Terrain::Mountain => tile_textures.mountain.clone(),
                    };

                    commands.spawn((
                        MapTile,
                        Sprite::from_image(texture),
                        Transform::from_xyz(world_x, world_y, 0.0)
                            .with_scale(Vec3::splat(scale)),
                        Visibility::Hidden,
                    ));
                }
            }
        }
    }

    commands.insert_resource(tile_textures);
    commands.insert_resource(MapDataResource::from(map_data));
}

pub fn spawn_player(
    mut commands: Commands,
    spawn_pos: Res<SpawnPosition>,
    asset_server: Res<AssetServer>,
) {
    let origin_x = -MAP_PIXEL_WIDTH / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -MAP_PIXEL_HEIGHT / 2.0 + TILE_SIZE / 2.0;

    let world_x = origin_x + spawn_pos.x as f32 * TILE_SIZE;
    let world_y = origin_y + spawn_pos.y as f32 * TILE_SIZE;

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

