use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_noise, save_image, TILE_SIZE};

pub fn generate_boss_cave_floor(output_dir: &Path) {
    let mut img = new_image();

    // 暗い紫系の床（瘴気が漂う感じ）
    let base = Rgba([90, 60, 95, 255]);
    let dark = Rgba([70, 45, 75, 255]);
    let light = Rgba([110, 75, 115, 255]);
    let miasma = Rgba([120, 55, 90, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r = pixel_noise(x, y, 200);
            let color = if r < 0.15 {
                dark
            } else if r < 0.25 {
                light
            } else if r < 0.30 {
                miasma
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    save_image(&img, output_dir, "boss_cave_floor.png");
}
