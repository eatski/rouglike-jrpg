use bevy::prelude::*;
use rand::Rng;

const MAP_WIDTH: usize = 100;
const MAP_HEIGHT: usize = 100;
const TILE_SIZE: f32 = 8.0;

#[derive(Clone, Copy)]
enum Terrain {
    Plains,
    Mountain,
    Forest,
    Sea,
}

impl Terrain {
    fn color(self) -> Color {
        match self {
            Terrain::Plains => Color::srgb_u8(120, 190, 120),
            Terrain::Mountain => Color::srgb_u8(139, 90, 43),
            Terrain::Forest => Color::srgb_u8(25, 110, 60),
            Terrain::Sea => Color::srgb_u8(40, 120, 220),
        }
    }
}

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
    let map_width = MAP_WIDTH as f32 * TILE_SIZE;
    let map_height = MAP_HEIGHT as f32 * TILE_SIZE;
    let origin_x = -map_width / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -map_height / 2.0 + TILE_SIZE / 2.0;
    let mut rng = rand::thread_rng();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let terrain = pick_terrain(&mut rng);
            let world_x = origin_x + x as f32 * TILE_SIZE;
            let world_y = origin_y + y as f32 * TILE_SIZE;

            commands.spawn((
                Sprite::from_color(terrain.color(), Vec2::splat(TILE_SIZE - 1.0)),
                Transform::from_xyz(world_x, world_y, 0.0),
            ));
        }
    }
}

fn pick_terrain(rng: &mut impl Rng) -> Terrain {
    let roll: f32 = rng.gen_range(0.0..1.0);

    if roll < 0.25 {
        Terrain::Sea
    } else if roll < 0.55 {
        Terrain::Forest
    } else if roll < 0.85 {
        Terrain::Plains
    } else {
        Terrain::Mountain
    }
}
