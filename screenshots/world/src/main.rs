use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;

use screenshot_common::{ScreenshotAppBuilder, ScreenshotRng};
use terrain::{Terrain, MAP_HEIGHT, MAP_WIDTH};
use world::map::generate_connected_map;

const TILE_SIZE: f32 = 4.0;

fn main() {
    let map_pixel = MAP_WIDTH as f32 * TILE_SIZE;

    let mut app = ScreenshotAppBuilder::new("world")
        .window_size(map_pixel as u32, map_pixel as u32)
        .build();
    app.add_systems(Startup, setup).run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut screenshot_rng: ResMut<ScreenshotRng>,
) {
    let map_pixel = MAP_WIDTH as f32 * TILE_SIZE;

    // カメラ: マップ全体を表示
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: map_pixel,
                height: map_pixel,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    // テクスチャをロード
    let tex_sea: Handle<Image> = asset_server.load("tiles/sea.png");
    let tex_plains: Handle<Image> = asset_server.load("tiles/plains.png");
    let tex_forest: Handle<Image> = asset_server.load("tiles/forest.png");
    let tex_mountain: Handle<Image> = asset_server.load("tiles/mountain.png");
    let tex_town: Handle<Image> = asset_server.load("tiles/town.png");
    let tex_cave: Handle<Image> = asset_server.load("tiles/cave.png");
    let tex_hokora: Handle<Image> = asset_server.load("tiles/hokora.png");

    // マップ生成（固定シード）
    let map_data = generate_connected_map(&mut screenshot_rng.rng);

    let scale = TILE_SIZE / 16.0;
    let origin_x = -(MAP_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -(MAP_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;

    // 全タイルをスプライトとして配置
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let terrain = map_data.grid[y][x];
            let texture = match terrain {
                Terrain::Sea => tex_sea.clone(),
                Terrain::Plains => tex_plains.clone(),
                Terrain::Forest => tex_forest.clone(),
                Terrain::Mountain => tex_mountain.clone(),
                Terrain::Town => tex_town.clone(),
                Terrain::Cave => tex_cave.clone(),
                Terrain::Hokora => tex_hokora.clone(),
                _ => tex_sea.clone(),
            };

            let world_x = origin_x + x as f32 * TILE_SIZE;
            let world_y = origin_y + y as f32 * TILE_SIZE;

            commands.spawn((
                Sprite::from_image(texture),
                Transform::from_xyz(world_x, world_y, 0.0).with_scale(Vec3::splat(scale)),
            ));
        }
    }
}
