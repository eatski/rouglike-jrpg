mod map;

use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};

use crate::map::{generate_map, MapData, Terrain, MAP_HEIGHT, MAP_WIDTH};

const TILE_SIZE: f32 = 4.0;
const MAP_PIXEL_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;
const MAP_PIXEL_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_SIZE;
const PLAYER_SIZE: f32 = TILE_SIZE * 0.7;
const VISIBLE_CELLS: f32 = 7.0;
const VISIBLE_SIZE: f32 = VISIBLE_CELLS * TILE_SIZE;
const WINDOW_SCALE: f32 = 16.0;
const WINDOW_SIZE: f32 = VISIBLE_SIZE * WINDOW_SCALE;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct TilePosition {
    x: usize,
    y: usize,
}

#[derive(Resource)]
struct SpawnPosition {
    x: usize,
    y: usize,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, (spawn_field_map, setup_camera, spawn_player).chain())
        .add_systems(Update, (player_movement, camera_follow).chain())
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: VISIBLE_SIZE,
                height: VISIBLE_SIZE,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn spawn_field_map(mut commands: Commands) {
    let origin_x = -MAP_PIXEL_WIDTH / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -MAP_PIXEL_HEIGHT / 2.0 + TILE_SIZE / 2.0;
    let mut rng = rand::thread_rng();
    let MapData { grid, spawn_position } = generate_map(&mut rng);

    commands.insert_resource(SpawnPosition {
        x: spawn_position.0,
        y: spawn_position.1,
    });

    // マップを3x3で複製描画（トーラスラップの視覚化）
    for offset_y in -1..=1 {
        for offset_x in -1..=1 {
            let base_x = origin_x + offset_x as f32 * MAP_PIXEL_WIDTH;
            let base_y = origin_y + offset_y as f32 * MAP_PIXEL_HEIGHT;

            for y in 0..MAP_HEIGHT {
                for x in 0..MAP_WIDTH {
                    let terrain = grid[y][x];
                    let world_x = base_x + x as f32 * TILE_SIZE;
                    let world_y = base_y + y as f32 * TILE_SIZE;

                    commands.spawn((
                        Sprite::from_color(terrain_color(terrain), Vec2::splat(TILE_SIZE)),
                        Transform::from_xyz(world_x, world_y, 0.0),
                    ));
                }
            }
        }
    }
}

fn spawn_player(mut commands: Commands, spawn_pos: Res<SpawnPosition>) {
    let origin_x = -MAP_PIXEL_WIDTH / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -MAP_PIXEL_HEIGHT / 2.0 + TILE_SIZE / 2.0;

    let world_x = origin_x + spawn_pos.x as f32 * TILE_SIZE;
    let world_y = origin_y + spawn_pos.y as f32 * TILE_SIZE;

    commands.spawn((
        Player,
        TilePosition {
            x: spawn_pos.x,
            y: spawn_pos.y,
        },
        Sprite::from_color(Color::srgb_u8(255, 200, 100), Vec2::splat(PLAYER_SIZE)),
        Transform::from_xyz(world_x, world_y, 1.0),
    ));
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut TilePosition, &mut Transform), With<Player>>,
) {
    let Ok((mut tile_pos, mut transform)) = query.single_mut() else {
        return;
    };

    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    if keyboard.just_pressed(KeyCode::KeyW) {
        dy = 1;
    }
    if keyboard.just_pressed(KeyCode::KeyS) {
        dy = -1;
    }
    if keyboard.just_pressed(KeyCode::KeyA) {
        dx = -1;
    }
    if keyboard.just_pressed(KeyCode::KeyD) {
        dx = 1;
    }

    if dx != 0 || dy != 0 {
        // タイル位置はラップ
        tile_pos.x = ((tile_pos.x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
        tile_pos.y = ((tile_pos.y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;

        // ワールド座標は連続的に移動
        transform.translation.x += dx as f32 * TILE_SIZE;
        transform.translation.y += dy as f32 * TILE_SIZE;

        // 中央マップの範囲を超えたらワールド座標をラップ
        let half_width = MAP_PIXEL_WIDTH / 2.0;
        let half_height = MAP_PIXEL_HEIGHT / 2.0;

        if transform.translation.x > half_width {
            transform.translation.x -= MAP_PIXEL_WIDTH;
        } else if transform.translation.x < -half_width {
            transform.translation.x += MAP_PIXEL_WIDTH;
        }

        if transform.translation.y > half_height {
            transform.translation.y -= MAP_PIXEL_HEIGHT;
        } else if transform.translation.y < -half_height {
            transform.translation.y += MAP_PIXEL_HEIGHT;
        }
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn terrain_color(terrain: Terrain) -> Color {
    match terrain {
        Terrain::Plains => Color::srgb_u8(120, 190, 120),
        Terrain::Mountain => Color::srgb_u8(139, 90, 43),
        Terrain::Forest => Color::srgb_u8(25, 110, 60),
        Terrain::Sea => Color::srgb_u8(40, 120, 220),
    }
}
