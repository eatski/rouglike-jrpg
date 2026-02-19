use image::{Rgba, RgbaImage};
use rand::Rng;
use std::path::Path;

use crate::generators::common::{new_image, save_image, TILE_SIZE};

#[allow(clippy::too_many_arguments)]
fn draw_house(
    img: &mut RgbaImage,
    _rng: &mut impl rand::Rng,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    _brick: Rgba<u8>,
    _brick_dark: Rgba<u8>,
    _brick_light: Rgba<u8>,
    roof_brown: Rgba<u8>,
    roof_highlight: Rgba<u8>,
    wall_beige: Rgba<u8>,
    wall_light: Rgba<u8>,
    wall_dark: Rgba<u8>,
    window: Rgba<u8>,
    door: Rgba<u8>,
) {
    let roof_height = 2;
    for dy in 0..roof_height {
        let y = start_y + dy;
        if y >= TILE_SIZE {
            break;
        }
        let roof_width = (roof_height - dy) * 2 + 1;
        let offset = (width - roof_width) / 2;
        for dx in 0..roof_width {
            let x = start_x + offset + dx;
            if x >= TILE_SIZE {
                continue;
            }
            let color = if dy == 0 { roof_highlight } else { roof_brown };
            img.put_pixel(x, y, color);
        }
    }

    let wall_start = start_y + roof_height;
    for dy in 0..(height - roof_height) {
        let y = wall_start + dy;
        if y >= TILE_SIZE {
            break;
        }
        for dx in 0..width {
            let x = start_x + dx;
            if x >= TILE_SIZE {
                continue;
            }
            let color = if dx == 0 {
                wall_dark
            } else if dx == width - 1 {
                wall_light
            } else {
                wall_beige
            };
            img.put_pixel(x, y, color);
        }
    }

    let window_y = wall_start + 1;
    if window_y < TILE_SIZE && start_x + 2 < TILE_SIZE {
        img.put_pixel(start_x + 2, window_y, window);
        img.put_pixel(start_x + 3, window_y, window);
    }

    let door_y_start = wall_start + (height - roof_height) - 3;
    for dy in 0..3 {
        let y = door_y_start + dy;
        if y >= TILE_SIZE {
            break;
        }
        if start_x + 2 < TILE_SIZE {
            img.put_pixel(start_x + 2, y, door);
        }
        if start_x + 3 < TILE_SIZE && dy < 2 {
            img.put_pixel(start_x + 3, y, door);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_castle(
    img: &mut RgbaImage,
    _rng: &mut impl rand::Rng,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    brick: Rgba<u8>,
    brick_dark: Rgba<u8>,
    brick_light: Rgba<u8>,
    _roof_brown: Rgba<u8>,
    _roof_highlight: Rgba<u8>,
    _wall_beige: Rgba<u8>,
    _wall_light: Rgba<u8>,
    _wall_dark: Rgba<u8>,
    window: Rgba<u8>,
) {
    for dx in 0..width {
        let x = start_x + dx;
        if x >= TILE_SIZE {
            continue;
        }
        let y = start_y;
        if y < TILE_SIZE {
            let color = if dx % 2 == 0 { brick_light } else { brick };
            img.put_pixel(x, y, color);
        }
    }

    for dy in 1..height {
        let y = start_y + dy;
        if y >= TILE_SIZE {
            break;
        }
        for dx in 0..width {
            let x = start_x + dx;
            if x >= TILE_SIZE {
                continue;
            }
            let brick_pattern = (dy + dx) % 3;
            let color = if dx == 0 || dy == height - 1 {
                brick_dark
            } else if dx == width - 1 {
                brick_light
            } else if brick_pattern == 0 {
                brick_dark
            } else {
                brick
            };
            img.put_pixel(x, y, color);
        }
    }

    let window_positions = [(start_x + 1, start_y + 3), (start_x + 3, start_y + 3), (start_x + 2, start_y + 6)];
    for (wx, wy) in window_positions {
        if wx < TILE_SIZE && wy < TILE_SIZE {
            img.put_pixel(wx, wy, window);
        }
    }
}

pub fn generate_town(output_dir: &Path) {
    let mut img = new_image();
    let mut rng = rand::thread_rng();

    let grass_base = Rgba([90, 140, 80, 255]);
    let grass_dark = Rgba([70, 110, 60, 255]);
    let brick = Rgba([160, 90, 70, 255]);
    let brick_dark = Rgba([120, 70, 50, 255]);
    let brick_light = Rgba([190, 110, 85, 255]);
    let roof_brown = Rgba([100, 60, 40, 255]);
    let roof_highlight = Rgba([140, 90, 60, 255]);
    let wall_beige = Rgba([220, 200, 170, 255]);
    let wall_light = Rgba([240, 230, 210, 255]);
    let wall_dark = Rgba([180, 160, 140, 255]);
    let window = Rgba([100, 120, 140, 255]);
    let door = Rgba([80, 50, 30, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let color = if rng.gen_bool(0.2) { grass_dark } else { grass_base };
            img.put_pixel(x, y, color);
        }
    }

    draw_house(&mut img, &mut rng, 1, 3, 5, 10, brick, brick_dark, brick_light, roof_brown, roof_highlight, wall_beige, wall_light, wall_dark, window, door);
    draw_castle(&mut img, &mut rng, 10, 2, 5, 11, brick, brick_dark, brick_light, roof_brown, roof_highlight, wall_beige, wall_light, wall_dark, window);

    let stone_light = Rgba([170, 170, 160, 255]);
    let stone_dark = Rgba([130, 130, 120, 255]);
    for y in 12..TILE_SIZE {
        for x in 6..=9 {
            let color = if rng.gen_bool(0.4) { stone_dark } else { stone_light };
            img.put_pixel(x, y, color);
        }
    }

    save_image(&img, output_dir, "town.png");
}
