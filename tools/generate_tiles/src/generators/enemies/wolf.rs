use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_wolf(output_dir: &Path) {
    let mut img = new_image();

    let fur_dark = Rgba([60, 60, 70, 255]);
    let fur_mid = Rgba([100, 100, 110, 255]);
    let fur_light = Rgba([140, 140, 150, 255]);
    let fur_white = Rgba([180, 180, 190, 255]);
    let nose_black = Rgba([30, 30, 35, 255]);
    let eye_yellow = Rgba([220, 200, 80, 255]);
    let eye_black = Rgba([0, 0, 0, 255]);
    let teeth = Rgba([230, 230, 220, 255]);

    // 耳
    img.put_pixel(4, 2, fur_dark);
    img.put_pixel(4, 3, fur_mid);
    img.put_pixel(5, 3, fur_dark);

    img.put_pixel(6, 2, fur_light);
    img.put_pixel(6, 3, fur_mid);
    img.put_pixel(7, 3, fur_light);

    // 頭上部
    for x in 4..=7 {
        let color = if x == 4 {
            fur_dark
        } else if x == 7 {
            fur_light
        } else {
            fur_mid
        };
        img.put_pixel(x, 4, color);
    }

    // 目
    img.put_pixel(3, 5, fur_mid);
    img.put_pixel(4, 5, fur_dark);
    img.put_pixel(5, 5, eye_yellow);
    img.put_pixel(6, 5, fur_light);
    img.put_pixel(7, 5, fur_light);

    // 瞳
    img.put_pixel(3, 6, fur_dark);
    img.put_pixel(4, 6, fur_mid);
    img.put_pixel(5, 6, eye_black);
    img.put_pixel(6, 6, fur_mid);
    img.put_pixel(7, 6, fur_white);

    // 鼻・口
    img.put_pixel(2, 7, fur_light);
    img.put_pixel(3, 7, fur_mid);
    img.put_pixel(4, 7, fur_mid);
    img.put_pixel(5, 7, fur_dark);
    img.put_pixel(6, 7, fur_white);
    img.put_pixel(7, 7, nose_black);

    // 口・牙
    img.put_pixel(2, 8, fur_dark);
    img.put_pixel(3, 8, teeth);
    img.put_pixel(4, 8, fur_dark);
    img.put_pixel(5, 8, fur_mid);
    img.put_pixel(6, 8, fur_light);

    // 首
    for x in 3..=8 {
        let color = if x == 3 || x == 4 {
            fur_dark
        } else if x == 7 || x == 8 {
            fur_light
        } else {
            fur_mid
        };
        img.put_pixel(x, 9, color);
    }

    // 体
    for x in 4..=12 {
        let color = if x <= 5 {
            fur_dark
        } else if x >= 11 {
            fur_light
        } else {
            fur_mid
        };
        img.put_pixel(x, 10, color);
    }

    for x in 5..=13 {
        let color = if x <= 6 {
            fur_dark
        } else if x >= 12 {
            fur_light
        } else {
            fur_mid
        };
        img.put_pixel(x, 11, color);
    }

    // 前脚
    img.put_pixel(10, 12, fur_mid);
    img.put_pixel(11, 12, fur_light);
    img.put_pixel(12, 12, fur_mid);

    img.put_pixel(10, 13, fur_dark);
    img.put_pixel(11, 13, fur_mid);
    img.put_pixel(12, 13, fur_dark);

    img.put_pixel(10, 14, fur_dark);
    img.put_pixel(12, 14, fur_dark);

    // 後脚
    img.put_pixel(5, 12, fur_dark);
    img.put_pixel(6, 12, fur_mid);
    img.put_pixel(7, 12, fur_dark);

    img.put_pixel(5, 13, fur_mid);
    img.put_pixel(6, 13, fur_dark);
    img.put_pixel(7, 13, fur_mid);

    img.put_pixel(5, 14, fur_dark);
    img.put_pixel(6, 14, fur_dark);

    // 尻尾
    img.put_pixel(13, 10, fur_mid);
    img.put_pixel(14, 10, fur_light);

    img.put_pixel(13, 11, fur_dark);
    img.put_pixel(14, 11, fur_mid);

    img.put_pixel(13, 12, fur_dark);

    save_image(&img, output_dir, "wolf.png");
}
