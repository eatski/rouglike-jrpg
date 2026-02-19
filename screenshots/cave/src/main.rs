use std::collections::HashMap;

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use app_state::OpenedChests;
use cave::{generate_cave_map, CAVE_HEIGHT, CAVE_WIDTH};
use cave_ui::{update_cave_tiles, CaveTilePool};
use movement_ui::{ActiveMap, Player, TilePosition, TILE_SIZE};
use screenshot_common::screenshot_app;
use world_ui::{camera_follow, load_tile_textures, setup_camera};

/// 洞窟マップを直接構築してActiveMapとして注入するシステム
fn setup_cave_direct(mut commands: Commands, asset_server: Res<AssetServer>) {
    // タイルテクスチャをロード
    let tile_textures = load_tile_textures(&asset_server);

    // 固定シードで洞窟マップを生成
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let cave_data = generate_cave_map(&mut rng);
    let (spawn_x, spawn_y) = cave_data.spawn_position;

    // 洞窟用ActiveMapを構築
    let cave_origin_x = -(CAVE_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let cave_origin_y = -(CAVE_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let active_map = ActiveMap {
        grid: cave_data.grid,
        width: cave_data.width,
        height: cave_data.height,
        origin_x: cave_origin_x,
        origin_y: cave_origin_y,
    };

    // プレイヤーをスポーン
    let player_texture: Handle<Image> = asset_server.load("characters/player.png");
    let scale = TILE_SIZE / 16.0;
    let (world_x, world_y) = active_map.to_world(spawn_x, spawn_y);
    commands.spawn((
        Player,
        TilePosition {
            x: spawn_x,
            y: spawn_y,
        },
        Sprite::from_image(player_texture),
        Transform::from_xyz(world_x, world_y, 1.0).with_scale(Vec3::splat(scale)),
    ));

    // 洞窟タイルプールを初期化
    commands.insert_resource(CaveTilePool {
        active_tiles: HashMap::new(),
        last_player_pos: None,
    });
    commands.insert_resource(tile_textures);
    commands.insert_resource(active_map);
}

fn main() {
    let mut app = screenshot_app("cave");
    app.init_resource::<OpenedChests>()
        .add_systems(Startup, (setup_cave_direct, setup_camera).chain())
        .add_systems(
            Update,
            (update_cave_tiles, camera_follow).chain(),
        )
        .run();
}
