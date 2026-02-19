use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_noise, save_image, TILE_SIZE};

pub fn generate_sea(output_dir: &Path) {
    let mut img = new_image();

    let base = Rgba([40, 80, 120, 255]);
    let light = Rgba([60, 120, 180, 255]);
    let highlight = Rgba([100, 160, 220, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let wave = ((y as f32 / 4.0).sin() * 2.0 + (x as f32 / 3.0).cos()) as i32;
            let color = if (y as i32 + wave) % 6 < 2 {
                light
            } else if pixel_noise(x, y, 1) < 0.05 {
                highlight
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    save_image(&img, output_dir, "sea.png");
}
