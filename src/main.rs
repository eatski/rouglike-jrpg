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
struct Bounce {
    direction: Vec2,
    timer: Timer,
    offset: Vec2,
}

const BOUNCE_DISTANCE: f32 = TILE_SIZE * 0.3;
const BOUNCE_DURATION: f32 = 0.12;

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

#[derive(Resource)]
struct MovementState {
    timer: Timer,
    initial_delay: Timer,
    is_repeating: bool,
    last_direction: (i32, i32),
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.08, TimerMode::Repeating),
            initial_delay: Timer::from_seconds(0.2, TimerMode::Once),
            is_repeating: false,
            last_direction: (0, 0),
        }
    }
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
        .init_resource::<MovementState>()
        .add_systems(Startup, (spawn_field_map, setup_camera, spawn_player).chain())
        .add_systems(Update, (player_movement, update_bounce, camera_follow).chain())
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
    let map_data = generate_map(&mut rng);

    commands.insert_resource(SpawnPosition {
        x: map_data.spawn_position.0,
        y: map_data.spawn_position.1,
    });

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

                    commands.spawn((
                        Sprite::from_color(terrain_color(terrain), Vec2::splat(TILE_SIZE)),
                        Transform::from_xyz(world_x, world_y, 0.0),
                    ));
                }
            }
        }
    }

    commands.insert_resource(map_data);
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
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map_data: Res<MapData>,
    mut move_state: ResMut<MovementState>,
    mut query: Query<(Entity, &mut TilePosition, &mut Transform, Option<&Bounce>), With<Player>>,
) {
    let Ok((entity, mut tile_pos, mut transform, bounce)) = query.single_mut() else {
        return;
    };

    // バウンス中は移動入力を無視
    if bounce.is_some() {
        return;
    }

    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        dy = 1;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dy = -1;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dx = -1;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dx = 1;
    }

    let current_direction = (dx, dy);

    // 方向キーが押されていない場合はリセット
    if dx == 0 && dy == 0 {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = (0, 0);
        return;
    }

    // 方向が変わったか判定（新しい入力として扱う）
    let direction_changed = current_direction != move_state.last_direction;

    let should_move = if direction_changed {
        // 方向変更時は即座に移動、タイマーリセット
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = current_direction;
        true
    } else if move_state.is_repeating {
        // リピート中は通常のタイマーで移動
        move_state.timer.tick(time.delta());
        move_state.timer.just_finished()
    } else {
        // 初回遅延を待つ
        move_state.initial_delay.tick(time.delta());
        if move_state.initial_delay.just_finished() {
            move_state.is_repeating = true;
            move_state.timer.reset();
            true
        } else {
            false
        }
    };

    if should_move {
        // 移動先のタイル座標を計算
        let new_x = ((tile_pos.x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
        let new_y = ((tile_pos.y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;

        // 海には移動できない
        if map_data.grid[new_y][new_x] == Terrain::Sea {
            // バウンスフィードバックを開始
            commands.entity(entity).insert(Bounce {
                direction: Vec2::new(dx as f32, dy as f32).normalize(),
                timer: Timer::from_seconds(BOUNCE_DURATION, TimerMode::Once),
                offset: Vec2::ZERO,
            });
            return;
        }

        // タイル位置を更新
        tile_pos.x = new_x;
        tile_pos.y = new_y;

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

fn update_bounce(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Bounce, &mut Transform), With<Player>>,
) {
    let Ok((entity, mut bounce, mut transform)) = query.single_mut() else {
        return;
    };

    // 前回のオフセットを元に戻す
    transform.translation.x -= bounce.offset.x;
    transform.translation.y -= bounce.offset.y;

    bounce.timer.tick(time.delta());

    if bounce.timer.just_finished() {
        // バウンス終了
        commands.entity(entity).remove::<Bounce>();
    } else {
        // バウンスアニメーション（往復）
        let progress = bounce.timer.fraction();
        // sin波で往復: 0→1→0
        let bounce_factor = (progress * std::f32::consts::PI).sin();
        let new_offset = bounce.direction * BOUNCE_DISTANCE * bounce_factor;

        transform.translation.x += new_offset.x;
        transform.translation.y += new_offset.y;
        bounce.offset = new_offset;
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
