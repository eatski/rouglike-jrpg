use image::Rgba;
use std::path::Path;

use crate::common::{new_image, save_image, TILE_SIZE};

pub fn generate_ladder(output_dir: &Path) {
    let mut img = new_image();

    let floor_base = Rgba([80, 75, 65, 255]);
    let wood_light = Rgba([180, 140, 80, 255]);
    let wood_mid = Rgba([140, 100, 55, 255]);
    let wood_dark = Rgba([100, 70, 35, 255]);
    let hole_dark = Rgba([30, 25, 20, 255]);
    let hole_mid = Rgba([50, 42, 35, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, floor_base);
        }
    }

    for y in 2..14 {
        for x in 4..12 {
            img.put_pixel(x, y, if x == 4 || x == 11 || y == 2 || y == 13 { hole_mid } else { hole_dark });
        }
    }

    let rail_left: u32 = 5;
    let rail_right: u32 = 10;
    for y in 1..15 {
        img.put_pixel(rail_left, y, wood_mid);
        img.put_pixel(rail_right, y, wood_mid);
    }

    let rungs = [3, 6, 9, 12];
    for &ry in &rungs {
        for x in (rail_left + 1)..rail_right {
            img.put_pixel(x, ry, wood_light);
        }
        if ry + 1 < TILE_SIZE {
            for x in (rail_left + 1)..rail_right {
                img.put_pixel(x, ry + 1, wood_dark);
            }
        }
    }

    save_image(&img, output_dir, "ladder.png");
}
