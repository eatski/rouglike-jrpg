mod map;

use bevy::prelude::*;

use crate::map::{generate_map, Terrain, MAP_HEIGHT, MAP_WIDTH};

const TILE_SIZE: f32 = 4.0;
const MAP_PIXEL_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;
const MAP_PIXEL_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_SIZE;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, spawn_field_map))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_field_map(mut commands: Commands) {
    let origin_x = -MAP_PIXEL_WIDTH / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -MAP_PIXEL_HEIGHT / 2.0 + TILE_SIZE / 2.0;
    let mut rng = rand::thread_rng();
    let map_data = generate_map(&mut rng);

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let terrain = map_data[y][x];
            let world_x = origin_x + x as f32 * TILE_SIZE;
            let world_y = origin_y + y as f32 * TILE_SIZE;

            commands.spawn((
                Sprite::from_color(terrain_color(terrain), Vec2::splat(TILE_SIZE)),
                Transform::from_xyz(world_x, world_y, 0.0),
            ));
        }
    }
}

fn terrain_color(terrain: Terrain) -> Color {
    match terrain {
        Terrain::Plains => Color::srgb_u8(120, 190, 120),
        Terrain::Mountain => Color::srgb_u8(139, 90, 43),
        Terrain::Forest => Color::srgb_u8(25, 110, 60),
        Terrain::Sea => Color::srgb_u8(40, 120, 220),
    }
}
