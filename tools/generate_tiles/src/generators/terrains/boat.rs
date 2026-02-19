use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_boat(output_dir: &Path) {
    let mut img = new_image();

    let wood_dark = Rgba([100, 60, 30, 255]);
    let wood_mid = Rgba([140, 90, 50, 255]);
    let wood_light = Rgba([180, 130, 80, 255]);
    let sail_white = Rgba([240, 240, 230, 255]);
    let sail_shadow = Rgba([200, 200, 190, 255]);
    let mast = Rgba([120, 80, 40, 255]);

    for y in 10..=14 {
        let row_width = match y {
            10 => (4, 12),
            11 => (3, 13),
            12 => (2, 14),
            13 => (2, 14),
            14 => (3, 13),
            _ => (4, 12),
        };
        for x in row_width.0..=row_width.1 {
            let color = if x <= 4 || y == 10 {
                wood_light
            } else if x >= 12 || y >= 13 {
                wood_dark
            } else {
                wood_mid
            };
            img.put_pixel(x, y, color);
        }
    }

    for x in 4..=12 {
        img.put_pixel(x, 9, wood_dark);
    }

    for y in 3..=9 {
        img.put_pixel(8, y, mast);
    }

    for y in 3..=8 {
        let sail_width = (y - 2).min(5);
        for dx in 1..=sail_width {
            let x = 8 - dx;
            if x >= 3 {
                let color = if dx == 1 { sail_shadow } else { sail_white };
                img.put_pixel(x, y, color);
            }
        }
    }

    for y in 4..=7 {
        let sail_width = ((y - 3) as i32).min(3);
        for dx in 1..=sail_width {
            let x = 8 + dx as u32;
            if x <= 11 {
                let color = if dx == 1 { sail_shadow } else { sail_white };
                img.put_pixel(x, y, color);
            }
        }
    }

    img.put_pixel(8, 2, Rgba([200, 50, 50, 255]));
    img.put_pixel(9, 2, Rgba([200, 50, 50, 255]));
    img.put_pixel(9, 1, Rgba([180, 40, 40, 255]));

    save_image(&img, output_dir, "boat.png");
}
