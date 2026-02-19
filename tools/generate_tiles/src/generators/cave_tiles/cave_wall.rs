use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_hash, pixel_noise, save_image, TILE_SIZE};

pub fn generate_cave_wall(output_dir: &Path) {
    let mut img = new_image();

    let dark = Rgba([30, 28, 35, 255]);
    let mid = Rgba([50, 45, 55, 255]);
    let light = Rgba([70, 65, 75, 255]);
    let highlight = Rgba([85, 80, 90, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r = pixel_noise(x, y, 110);
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

    let crack_positions: [(u32, u32); 4] = [
        (2 + pixel_hash(0, 0, 111) % (TILE_SIZE - 4), 2 + pixel_hash(0, 1, 111) % (TILE_SIZE - 4)),
        (2 + pixel_hash(1, 0, 112) % (TILE_SIZE - 4), 2 + pixel_hash(1, 1, 112) % (TILE_SIZE - 4)),
        (2 + pixel_hash(2, 0, 113) % (TILE_SIZE - 4), 2 + pixel_hash(2, 1, 113) % (TILE_SIZE - 4)),
        (2 + pixel_hash(3, 0, 114) % (TILE_SIZE - 4), 2 + pixel_hash(3, 1, 114) % (TILE_SIZE - 4)),
    ];
    for (cx, cy) in crack_positions {
        img.put_pixel(cx, cy, Rgba([20, 18, 25, 255]));
        if cx + 1 < TILE_SIZE {
            img.put_pixel(cx + 1, cy, Rgba([25, 22, 30, 255]));
        }
    }

    save_image(&img, output_dir, "cave_wall.png");
}
