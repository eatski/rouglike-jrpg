use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_scorpion(output_dir: &Path) {
    let mut img = new_image();

    let body_dark = Rgba([100, 30, 40, 255]);
    let body_mid = Rgba([140, 50, 60, 255]);
    let body_light = Rgba([170, 70, 80, 255]);
    let claw = Rgba([120, 40, 50, 255]);
    let claw_tip = Rgba([160, 60, 70, 255]);
    let stinger = Rgba([200, 180, 50, 255]);
    let stinger_tip = Rgba([180, 255, 80, 255]);
    let eye = Rgba([255, 220, 80, 255]);
    let leg = Rgba([90, 30, 35, 255]);

    // 毒針先端 (尻尾の上部)
    img.put_pixel(13, 1, stinger_tip);
    img.put_pixel(14, 1, stinger_tip);

    // 尻尾 (右上から丸まっている)
    img.put_pixel(12, 2, stinger);
    img.put_pixel(13, 2, stinger);
    img.put_pixel(14, 2, body_dark);

    img.put_pixel(11, 3, body_mid);
    img.put_pixel(12, 3, body_mid);

    img.put_pixel(11, 4, body_mid);
    img.put_pixel(12, 4, body_dark);

    img.put_pixel(11, 5, body_dark);
    img.put_pixel(12, 5, body_mid);

    // 尻尾と体の接続
    img.put_pixel(10, 6, body_mid);
    img.put_pixel(11, 6, body_dark);

    // 体 (横に広い)
    for x in 5..=10 {
        let color = if x == 5 || x == 10 {
            body_dark
        } else if x == 6 || x == 9 {
            body_mid
        } else {
            body_light
        };
        img.put_pixel(x, 7, color);
    }

    // 体中央 (頭部側)
    for x in 4..=9 {
        let color = if x == 4 || x == 9 {
            body_dark
        } else if x == 5 || x == 8 {
            body_mid
        } else {
            body_light
        };
        img.put_pixel(x, 8, color);
    }

    // 頭部
    for x in 4..=8 {
        let color = if x == 4 || x == 8 {
            body_dark
        } else {
            body_mid
        };
        img.put_pixel(x, 9, color);
    }

    // 目
    img.put_pixel(5, 9, eye);
    img.put_pixel(7, 9, eye);

    // 口
    img.put_pixel(5, 10, body_dark);
    img.put_pixel(6, 10, body_mid);
    img.put_pixel(7, 10, body_dark);

    // ハサミ (左)
    img.put_pixel(1, 5, claw_tip);
    img.put_pixel(2, 5, claw);
    img.put_pixel(1, 6, claw_tip);
    img.put_pixel(2, 6, claw);
    img.put_pixel(3, 6, claw);
    img.put_pixel(3, 7, claw);
    img.put_pixel(4, 7, body_dark);

    // ハサミ (右)
    img.put_pixel(13, 6, claw_tip);
    img.put_pixel(14, 6, claw_tip);
    img.put_pixel(12, 7, claw);
    img.put_pixel(13, 7, claw);
    img.put_pixel(14, 7, claw);
    img.put_pixel(11, 7, body_dark);
    img.put_pixel(12, 6, claw);

    // 脚 (左側4本)
    img.put_pixel(3, 8, leg);
    img.put_pixel(2, 9, leg);

    img.put_pixel(3, 9, leg);
    img.put_pixel(2, 10, leg);

    img.put_pixel(4, 10, leg);
    img.put_pixel(3, 11, leg);

    img.put_pixel(4, 11, leg);
    img.put_pixel(3, 12, leg);

    // 脚 (右側4本)
    img.put_pixel(9, 8, leg);
    img.put_pixel(10, 9, leg);

    img.put_pixel(9, 9, leg);
    img.put_pixel(10, 10, leg);

    img.put_pixel(8, 10, leg);
    img.put_pixel(9, 11, leg);

    img.put_pixel(8, 11, leg);
    img.put_pixel(9, 12, leg);

    save_image(&img, output_dir, "scorpion.png");
}
