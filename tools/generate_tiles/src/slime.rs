use image::Rgba;
use std::path::Path;

use crate::common::{new_image, save_image};

pub fn generate_slime(output_dir: &Path) {
    let mut img = new_image();

    let slime_dark = Rgba([40, 100, 180, 255]);
    let slime_mid = Rgba([60, 140, 220, 255]);
    let slime_light = Rgba([100, 180, 240, 255]);
    let slime_bright = Rgba([150, 220, 255, 255]);
    let eye_white = Rgba([255, 255, 255, 255]);
    let eye_black = Rgba([0, 0, 0, 255]);

    // 最上部
    for x in 6..=9 {
        img.put_pixel(x, 4, slime_dark);
    }
    for x in 7..=8 {
        img.put_pixel(x, 5, slime_mid);
    }

    // 上部
    for x in 5..=10 {
        let color = if x == 5 || x == 10 {
            slime_dark
        } else if x == 6 || x == 9 {
            slime_mid
        } else {
            slime_light
        };
        img.put_pixel(x, 6, color);
    }

    // 目
    img.put_pixel(5, 7, slime_dark);
    img.put_pixel(6, 7, eye_white);
    img.put_pixel(7, 7, slime_light);
    img.put_pixel(8, 7, slime_light);
    img.put_pixel(9, 7, eye_white);
    img.put_pixel(10, 7, slime_dark);

    // 瞳
    img.put_pixel(4, 8, slime_dark);
    img.put_pixel(5, 8, slime_mid);
    img.put_pixel(6, 8, eye_black);
    img.put_pixel(7, 8, slime_light);
    img.put_pixel(8, 8, slime_light);
    img.put_pixel(9, 8, eye_black);
    img.put_pixel(10, 8, slime_mid);
    img.put_pixel(11, 8, slime_dark);

    // 中央部
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            slime_dark
        } else if x == 5 || x == 10 {
            slime_mid
        } else if x == 7 || x == 8 {
            slime_bright
        } else {
            slime_light
        };
        img.put_pixel(x, 9, color);
    }
    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            slime_dark
        } else if x == 4 || x == 11 {
            slime_mid
        } else if x == 7 || x == 8 {
            slime_bright
        } else {
            slime_light
        };
        img.put_pixel(x, 10, color);
    }

    // 下部
    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            slime_dark
        } else if x == 4 || x == 11 {
            slime_mid
        } else {
            slime_light
        };
        img.put_pixel(x, 11, color);
    }
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            slime_dark
        } else if x == 5 || x == 10 {
            slime_mid
        } else {
            slime_light
        };
        img.put_pixel(x, 12, color);
    }

    // 底部
    for x in 5..=10 {
        let color = if x == 5 || x == 10 {
            slime_dark
        } else {
            slime_mid
        };
        img.put_pixel(x, 13, color);
    }
    for x in 6..=9 {
        img.put_pixel(x, 14, slime_dark);
    }

    // ハイライト
    img.put_pixel(6, 6, slime_bright);
    img.put_pixel(7, 6, slime_bright);
    img.put_pixel(6, 9, slime_bright);
    img.put_pixel(9, 10, slime_bright);

    save_image(&img, output_dir, "slime.png");
}
