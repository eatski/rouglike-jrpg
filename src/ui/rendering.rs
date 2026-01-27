use bevy::prelude::*;

use crate::game::map::{generate_map, Terrain, MAP_HEIGHT, MAP_WIDTH};
use crate::game::movement::{Player, SpawnPosition, TilePosition};

use super::constants::{MAP_PIXEL_HEIGHT, MAP_PIXEL_WIDTH, PLAYER_SIZE, TILE_SIZE};

pub fn spawn_field_map(mut commands: Commands) {
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

pub fn spawn_player(mut commands: Commands, spawn_pos: Res<SpawnPosition>) {
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

fn terrain_color(terrain: Terrain) -> Color {
    match terrain {
        Terrain::Plains => Color::srgb_u8(120, 190, 120),
        Terrain::Mountain => Color::srgb_u8(139, 90, 43),
        Terrain::Forest => Color::srgb_u8(25, 110, 60),
        Terrain::Sea => Color::srgb_u8(40, 120, 220),
    }
}
