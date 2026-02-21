use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_noise, save_image, TILE_SIZE};

pub fn generate_dark_plains(output_dir: &Path) {
    let mut img = new_image();

    let base = Rgba([90, 70, 100, 255]);
    let dark = Rgba([60, 45, 75, 255]);
    let light = Rgba([110, 85, 115, 255]);
    let toxic = Rgba([120, 90, 50, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r = pixel_noise(x, y, 200);
            let color = if r < 0.15 {
                dark
            } else if r < 0.25 {
                light
            } else if r < 0.28 {
                toxic
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    save_image(&img, output_dir, "dark_plains.png");
}
