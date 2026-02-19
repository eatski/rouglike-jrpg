use image::{Rgba, RgbaImage};
use rand::Rng;
use std::path::Path;

use crate::generators::common::{new_image, save_image, TILE_SIZE};

struct RockColors {
    dark: Rgba<u8>,
    mid: Rgba<u8>,
    light: Rgba<u8>,
    edge: Rgba<u8>,
    moss: Rgba<u8>,
}

fn draw_rocky_boulder(
    img: &mut RgbaImage,
    rng: &mut impl rand::Rng,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    colors: &RockColors,
) {
    let RockColors { dark: rock_dark, mid: rock_mid, light: rock_light, edge: rock_edge, moss } = *colors;
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

            let noise = rng.gen_range(0..3);
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
                if rng.gen_bool(0.6) { rock_light } else { rock_mid }
            } else {
                let r: f32 = rng.r#gen();
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
        }
    }

    for _ in 0..3 {
        let cx = start_x + rng.gen_range(1..width.saturating_sub(1));
        let cy = start_y + rng.gen_range(1..height.saturating_sub(1));
        if cx < TILE_SIZE && cy < TILE_SIZE {
            img.put_pixel(cx, cy, rock_edge);
        }
    }
}

pub fn generate_mountain(output_dir: &Path) {
    let mut img = new_image();
    let mut rng = rand::thread_rng();

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

    draw_rocky_boulder(&mut img, &mut rng, 6, 5, 8, 10, &rock_colors);
    draw_rocky_boulder(&mut img, &mut rng, 1, 2, 5, 6, &rock_colors);
    draw_rocky_boulder(&mut img, &mut rng, 11, 3, 4, 5, &rock_colors);

    for _ in 0..8 {
        let x = rng.gen_range(0..TILE_SIZE);
        let y = rng.gen_range(10..TILE_SIZE);
        let color = if rng.gen_bool(0.5) { rock_mid } else { rock_dark };
        if img.get_pixel(x, y) == &bg {
            img.put_pixel(x, y, color);
        }
    }

    save_image(&img, output_dir, "mountain.png");
}
