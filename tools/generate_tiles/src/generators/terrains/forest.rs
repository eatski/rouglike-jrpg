use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_noise, save_image, TILE_SIZE};

pub fn generate_forest(output_dir: &Path) {
    let mut img = new_image();

    let base = Rgba([25, 100, 50, 255]);
    let dark = Rgba([15, 70, 35, 255]);
    let light = Rgba([45, 130, 70, 255]);
    let trunk = Rgba([80, 50, 30, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, base);
        }
    }

    let tree_positions = [(4, 8), (11, 6), (7, 13)];
    for (i, (tx, ty)) in tree_positions.iter().enumerate() {
        let (tx, ty) = (*tx, *ty);
        if ty < TILE_SIZE && tx < TILE_SIZE {
            img.put_pixel(tx, ty, trunk);
            if ty + 1 < TILE_SIZE {
                img.put_pixel(tx, ty + 1, trunk);
            }
        }
        for dy in 0..4u32 {
            let leaf_y = ty.saturating_sub(dy + 1);
            let width = (dy + 1).min(3);
            for dx in 0..=width {
                let lx1 = tx.saturating_sub(dx / 2);
                let lx2 = (tx + dx / 2).min(TILE_SIZE - 1);
                if leaf_y < TILE_SIZE {
                    if lx1 < TILE_SIZE {
                        let c = if pixel_noise(lx1, leaf_y, 10 + i as u32) < 0.3 {
                            light
                        } else {
                            dark
                        };
                        img.put_pixel(lx1, leaf_y, c);
                    }
                    if lx2 < TILE_SIZE && lx2 != lx1 {
                        let c = if pixel_noise(lx2, leaf_y, 20 + i as u32) < 0.3 {
                            light
                        } else {
                            dark
                        };
                        img.put_pixel(lx2, leaf_y, c);
                    }
                }
            }
        }
    }

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            if img.get_pixel(x, y) == &base && pixel_noise(x, y, 30) < 0.2 {
                let c = if pixel_noise(x, y, 31) < 0.5 { dark } else { light };
                img.put_pixel(x, y, c);
            }
        }
    }

    save_image(&img, output_dir, "forest.png");
}
