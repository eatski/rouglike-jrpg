use std::path::Path;

use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowResolution;

use terrain::{Terrain, MAP_HEIGHT, MAP_WIDTH};
use world::map::generate_connected_map;

const TILE_SIZE: f32 = 4.0;
const OUTPUT_PATH: &str = "screenshots/output/world.png";

fn main() {
    let map_pixel = MAP_WIDTH as f32 * TILE_SIZE; // 600.0

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: "../../assets".to_string(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Screenshot: world".to_string(),
                    resolution: WindowResolution::new(map_pixel as u32, map_pixel as u32),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(FrameCounter(0))
    .add_systems(Startup, setup)
    .add_systems(Update, screenshot_system)
    .run();
}

#[derive(Resource)]
struct FrameCounter(u32);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    // マップ生成
    let mut rng = rand::thread_rng();
    let map_data = generate_connected_map(&mut rng);

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

fn screenshot_system(
    mut commands: Commands,
    mut counter: ResMut<FrameCounter>,
    mut exit: MessageWriter<AppExit>,
) {
    counter.0 += 1;
    if counter.0 == 30 {
        if let Some(parent) = Path::new(OUTPUT_PATH).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(OUTPUT_PATH.to_string()));
    }
    if counter.0 == 40 {
        exit.write(AppExit::Success);
    }
}
