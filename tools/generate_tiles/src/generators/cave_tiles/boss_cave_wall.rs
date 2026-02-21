use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_hash, pixel_noise, save_image, TILE_SIZE};

pub fn generate_boss_cave_wall(output_dir: &Path) {
    let mut img = new_image();

    // 紫・暗赤系パレット（瘴気の洞窟）
    let dark = Rgba([22, 10, 28, 255]);
    let mid = Rgba([42, 18, 48, 255]);
    let light = Rgba([60, 28, 65, 255]);
    let highlight = Rgba([78, 35, 80, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r = pixel_noise(x, y, 210);
            let color = if r < 0.3 {
                dark
            } else if r < 0.6 {
                mid
            } else if r < 0.9 {
                light
            } else {
                highlight
            };
            img.put_pixel(x, y, color);
        }
    }

    // ひび割れ
    let crack_positions: [(u32, u32); 4] = [
        (2 + pixel_hash(0, 0, 211) % (TILE_SIZE - 4), 2 + pixel_hash(0, 1, 211) % (TILE_SIZE - 4)),
        (2 + pixel_hash(1, 0, 212) % (TILE_SIZE - 4), 2 + pixel_hash(1, 1, 212) % (TILE_SIZE - 4)),
        (2 + pixel_hash(2, 0, 213) % (TILE_SIZE - 4), 2 + pixel_hash(2, 1, 213) % (TILE_SIZE - 4)),
        (2 + pixel_hash(3, 0, 214) % (TILE_SIZE - 4), 2 + pixel_hash(3, 1, 214) % (TILE_SIZE - 4)),
    ];
    for (cx, cy) in crack_positions {
        img.put_pixel(cx, cy, Rgba([25, 8, 30, 255]));
        if cx + 1 < TILE_SIZE {
            img.put_pixel(cx + 1, cy, Rgba([30, 12, 35, 255]));
        }
    }

    save_image(&img, output_dir, "boss_cave_wall.png");
}
