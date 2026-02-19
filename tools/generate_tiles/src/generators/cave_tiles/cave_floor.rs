use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_noise, save_image, TILE_SIZE};

pub fn generate_cave_floor(output_dir: &Path) {
    let mut img = new_image();

    let base = Rgba([130, 120, 100, 255]);
    let dark = Rgba([105, 96, 80, 255]);
    let light = Rgba([155, 143, 120, 255]);
    let pebble = Rgba([170, 158, 132, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r = pixel_noise(x, y, 100);
            let color = if r < 0.15 {
                dark
            } else if r < 0.25 {
                light
            } else if r < 0.30 {
                pebble
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    save_image(&img, output_dir, "cave_floor.png");
}
