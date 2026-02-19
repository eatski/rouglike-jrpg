use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_noise, save_image, TILE_SIZE};

pub fn generate_cave_floor(output_dir: &Path) {
    let mut img = new_image();

    let base = Rgba([80, 75, 65, 255]);
    let dark = Rgba([60, 55, 48, 255]);
    let light = Rgba([100, 95, 82, 255]);
    let pebble = Rgba([110, 105, 90, 255]);

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
