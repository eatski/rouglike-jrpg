use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image, TILE_SIZE};

pub fn generate_warp_zone(output_dir: &Path) {
    let mut img = new_image();

    let base = Rgba([60, 55, 48, 255]);
    let glow_outer = Rgba([80, 60, 160, 255]);
    let glow_mid = Rgba([120, 90, 220, 255]);
    let glow_inner = Rgba([180, 150, 255, 255]);
    let glow_center = Rgba([220, 210, 255, 255]);

    let cx = TILE_SIZE as f32 / 2.0;
    let cy = TILE_SIZE as f32 / 2.0;
    let max_radius = 7.0f32;

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let dx = x as f32 - cx + 0.5;
            let dy = y as f32 - cy + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();

            let color = if dist < 2.0 {
                glow_center
            } else if dist < 3.5 {
                glow_inner
            } else if dist < 5.0 {
                glow_mid
            } else if dist < max_radius {
                glow_outer
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    save_image(&img, output_dir, "warp_zone.png");
}
