use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_ghost(output_dir: &Path) {
    let mut img = new_image();

    let ghost_white = Rgba([240, 240, 250, 255]);
    let ghost_light = Rgba([220, 220, 240, 255]);
    let ghost_mid = Rgba([200, 200, 230, 255]);
    let ghost_dark = Rgba([180, 180, 220, 255]);
    let ghost_outline = Rgba([160, 160, 210, 255]);
    let eye_red = Rgba([200, 50, 50, 255]);
    let eye_glow = Rgba([255, 100, 100, 255]);

    // 頭部
    for x in 7..=8 {
        img.put_pixel(x, 2, ghost_outline);
    }

    for x in 6..=9 {
        let color = if x == 6 || x == 9 { ghost_outline } else { ghost_white };
        img.put_pixel(x, 3, color);
    }

    for x in 5..=10 {
        let color = if x == 5 || x == 10 {
            ghost_outline
        } else if x == 6 || x == 9 {
            ghost_mid
        } else {
            ghost_white
        };
        img.put_pixel(x, 4, color);
    }

    // 目
    img.put_pixel(4, 5, ghost_outline);
    img.put_pixel(5, 5, ghost_dark);
    img.put_pixel(6, 5, eye_glow);
    img.put_pixel(7, 5, ghost_light);
    img.put_pixel(8, 5, ghost_light);
    img.put_pixel(9, 5, eye_glow);
    img.put_pixel(10, 5, ghost_dark);
    img.put_pixel(11, 5, ghost_outline);

    // 瞳
    img.put_pixel(4, 6, ghost_dark);
    img.put_pixel(5, 6, ghost_mid);
    img.put_pixel(6, 6, eye_red);
    img.put_pixel(7, 6, ghost_white);
    img.put_pixel(8, 6, ghost_white);
    img.put_pixel(9, 6, eye_red);
    img.put_pixel(10, 6, ghost_mid);
    img.put_pixel(11, 6, ghost_dark);

    // 口
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            ghost_dark
        } else if (6..=9).contains(&x) {
            ghost_outline
        } else {
            ghost_mid
        };
        img.put_pixel(x, 7, color);
    }

    // 体（上半身）
    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            ghost_outline
        } else if x == 4 || x == 11 {
            ghost_dark
        } else if x == 5 || x == 10 {
            ghost_mid
        } else {
            ghost_white
        };
        img.put_pixel(x, 8, color);
    }

    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            ghost_dark
        } else if x == 4 || x == 11 {
            ghost_mid
        } else if x == 7 || x == 8 {
            ghost_light
        } else {
            ghost_white
        };
        img.put_pixel(x, 9, color);
    }

    for x in 2..=13 {
        let color = if x == 2 || x == 13 {
            ghost_outline
        } else if x == 3 || x == 12 {
            ghost_dark
        } else if x == 7 || x == 8 {
            ghost_light
        } else {
            ghost_mid
        };
        img.put_pixel(x, 10, color);
    }

    // 下半身（波打つ裾）
    for x in 2..=13 {
        let color = if x == 2 || x == 13 { ghost_dark } else { ghost_mid };
        img.put_pixel(x, 11, color);
    }

    for x in [3, 4, 7, 8, 11, 12] {
        img.put_pixel(x, 12, ghost_dark);
    }

    for x in [4, 8, 12] {
        img.put_pixel(x, 13, ghost_outline);
    }

    for x in [5, 9] {
        img.put_pixel(x, 14, ghost_outline);
    }

    // 浮遊感ハイライト
    img.put_pixel(4, 8, ghost_light);
    img.put_pixel(11, 8, ghost_light);
    img.put_pixel(5, 9, ghost_white);
    img.put_pixel(10, 9, ghost_white);

    save_image(&img, output_dir, "ghost.png");
}
