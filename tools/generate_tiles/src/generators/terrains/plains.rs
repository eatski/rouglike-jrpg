use image::Rgba;
use rand::Rng;
use std::path::Path;

use crate::generators::common::{new_image, save_image, TILE_SIZE};

pub fn generate_plains(output_dir: &Path) {
    let mut img = new_image();
    let mut rng = rand::thread_rng();

    let base = Rgba([120, 180, 100, 255]);
    let dark = Rgba([80, 140, 70, 255]);
    let light = Rgba([150, 210, 120, 255]);
    let flower = Rgba([220, 180, 80, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r: f32 = rng.r#gen();
            let color = if r < 0.15 {
                dark
            } else if r < 0.25 {
                light
            } else if r < 0.28 {
                flower
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    save_image(&img, output_dir, "plains.png");
}
