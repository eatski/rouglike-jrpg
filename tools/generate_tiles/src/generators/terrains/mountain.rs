use image::{Rgba, RgbaImage};
use std::path::Path;

use crate::generators::common::{new_image, pixel_hash, pixel_noise, save_image, TILE_SIZE};

struct RockColors {
    dark: Rgba<u8>,
    mid: Rgba<u8>,
    light: Rgba<u8>,
    edge: Rgba<u8>,
    moss: Rgba<u8>,
}

fn draw_rocky_boulder(
    img: &mut RgbaImage,
    salt: u32,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    colors: &RockColors,
) {
    let RockColors { dark: rock_dark, mid: rock_mid, light: rock_light, edge: rock_edge, moss } = *colors;
    let mut counter: u32 = 0;
    for dy in 0..height {
        let y = start_y + dy;
        if y >= TILE_SIZE {
            break;
        }

        let progress = dy as f32 / height as f32;
        let bulge = if progress < 0.3 {
            (progress / 0.3 * 0.8 + 0.2) * width as f32
        } else if progress < 0.7 {
            width as f32
        } else {
            ((1.0 - progress) / 0.3 * 0.6 + 0.4) * width as f32
        };

        let current_width = bulge as u32;
        let offset = (width - current_width) / 2;

        for dx in 0..current_width {
            let x = start_x + offset + dx;
            if x >= TILE_SIZE {
                continue;
            }

            let noise = pixel_hash(x, y, salt) % 3;
            if noise == 0 && dx == 0 {
                continue;
            }
            if noise == 1 && dx == current_width - 1 {
                continue;
            }

            let is_left_edge = dx == 0;
            let is_right_edge = dx == current_width - 1;
            let is_top = dy < 2;
            let is_bottom = dy >= height - 2;

            let color = if is_left_edge || is_bottom {
                rock_edge
            } else if is_right_edge || is_top {
                if pixel_noise(x, y, salt + 1) < 0.6 { rock_light } else { rock_mid }
            } else {
                let r = pixel_noise(x, y, salt + 2);
                if r < 0.1 {
                    moss
                } else if r < 0.3 {
                    rock_light
                } else if r < 0.6 {
                    rock_mid
                } else {
                    rock_dark
                }
            };

            img.put_pixel(x, y, color);
            counter += 1;
        }
    }

    let crack_positions: [(u32, u32); 3] = [
        (start_x + pixel_hash(0, counter, salt + 3) % width.saturating_sub(2).max(1) + 1,
         start_y + pixel_hash(1, counter, salt + 3) % height.saturating_sub(2).max(1) + 1),
        (start_x + pixel_hash(2, counter, salt + 4) % width.saturating_sub(2).max(1) + 1,
         start_y + pixel_hash(3, counter, salt + 4) % height.saturating_sub(2).max(1) + 1),
        (start_x + pixel_hash(4, counter, salt + 5) % width.saturating_sub(2).max(1) + 1,
         start_y + pixel_hash(5, counter, salt + 5) % height.saturating_sub(2).max(1) + 1),
    ];
    for (cx, cy) in crack_positions {
        if cx < TILE_SIZE && cy < TILE_SIZE {
            img.put_pixel(cx, cy, rock_edge);
        }
    }
}

pub fn generate_mountain(output_dir: &Path) {
    let mut img = new_image();

    let bg = Rgba([80, 100, 70, 255]);
    let rock_dark = Rgba([80, 70, 60, 255]);
    let rock_mid = Rgba([120, 110, 100, 255]);
    let rock_light = Rgba([160, 150, 135, 255]);
    let rock_edge = Rgba([60, 55, 50, 255]);
    let moss = Rgba([70, 90, 60, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, bg);
        }
    }

    let rock_colors = RockColors { dark: rock_dark, mid: rock_mid, light: rock_light, edge: rock_edge, moss };

    draw_rocky_boulder(&mut img, 50, 6, 5, 8, 10, &rock_colors);
    draw_rocky_boulder(&mut img, 60, 1, 2, 5, 6, &rock_colors);
    draw_rocky_boulder(&mut img, 70, 11, 3, 4, 5, &rock_colors);

    let scatter_positions: [(u32, u32); 8] = [
        (pixel_hash(0, 0, 80) % TILE_SIZE, 10 + pixel_hash(0, 1, 80) % (TILE_SIZE - 10)),
        (pixel_hash(1, 0, 80) % TILE_SIZE, 10 + pixel_hash(1, 1, 80) % (TILE_SIZE - 10)),
        (pixel_hash(2, 0, 80) % TILE_SIZE, 10 + pixel_hash(2, 1, 80) % (TILE_SIZE - 10)),
        (pixel_hash(3, 0, 80) % TILE_SIZE, 10 + pixel_hash(3, 1, 80) % (TILE_SIZE - 10)),
        (pixel_hash(4, 0, 80) % TILE_SIZE, 10 + pixel_hash(4, 1, 80) % (TILE_SIZE - 10)),
        (pixel_hash(5, 0, 80) % TILE_SIZE, 10 + pixel_hash(5, 1, 80) % (TILE_SIZE - 10)),
        (pixel_hash(6, 0, 80) % TILE_SIZE, 10 + pixel_hash(6, 1, 80) % (TILE_SIZE - 10)),
        (pixel_hash(7, 0, 80) % TILE_SIZE, 10 + pixel_hash(7, 1, 80) % (TILE_SIZE - 10)),
    ];
    for (x, y) in scatter_positions {
        let color = if pixel_noise(x, y, 81) < 0.5 { rock_mid } else { rock_dark };
        if img.get_pixel(x, y) == &bg {
            img.put_pixel(x, y, color);
        }
    }

    save_image(&img, output_dir, "mountain.png");
}
