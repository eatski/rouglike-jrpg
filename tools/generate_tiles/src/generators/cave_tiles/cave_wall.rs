use image::Rgba;
use rand::Rng;
use std::path::Path;

use crate::generators::common::{new_image, save_image, TILE_SIZE};

pub fn generate_cave_wall(output_dir: &Path) {
    let mut img = new_image();
    let mut rng = rand::thread_rng();

    let dark = Rgba([30, 28, 35, 255]);
    let mid = Rgba([50, 45, 55, 255]);
    let light = Rgba([70, 65, 75, 255]);
    let highlight = Rgba([85, 80, 90, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r: f32 = rng.r#gen();
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

    for _ in 0..4 {
        let cx = rng.gen_range(2..TILE_SIZE - 2);
        let cy = rng.gen_range(2..TILE_SIZE - 2);
        img.put_pixel(cx, cy, Rgba([20, 18, 25, 255]));
        if cx + 1 < TILE_SIZE {
            img.put_pixel(cx + 1, cy, Rgba([25, 22, 30, 255]));
        }
    }

    save_image(&img, output_dir, "cave_wall.png");
}
