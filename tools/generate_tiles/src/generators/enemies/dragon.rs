use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_dragon(output_dir: &Path) {
    let mut img = new_image();

    let scale_dark = Rgba([140, 30, 20, 255]);
    let scale_mid = Rgba([190, 60, 30, 255]);
    let scale_light = Rgba([220, 90, 40, 255]);
    let scale_bright = Rgba([240, 130, 50, 255]);
    let belly = Rgba([230, 190, 100, 255]);
    let eye_yellow = Rgba([255, 240, 60, 255]);
    let eye_slit = Rgba([30, 10, 5, 255]);
    let wing_dark = Rgba([120, 25, 15, 255]);
    let wing_mid = Rgba([160, 40, 25, 255]);
    let fire_red = Rgba([255, 80, 20, 255]);
    let fire_orange = Rgba([255, 160, 40, 255]);
    let fire_yellow = Rgba([255, 230, 80, 255]);
    let horn = Rgba([180, 160, 100, 255]);

    // 角
    img.put_pixel(5, 0, horn);
    img.put_pixel(9, 0, horn);
    img.put_pixel(6, 1, horn);
    img.put_pixel(9, 1, horn);

    // 頭部
    for x in 6..=9 {
        img.put_pixel(x, 2, scale_dark);
    }
    for x in 5..=10 {
        let color = if x == 5 || x == 10 { scale_dark } else { scale_mid };
        img.put_pixel(x, 3, color);
    }

    // 目
    img.put_pixel(5, 4, scale_dark);
    img.put_pixel(6, 4, eye_yellow);
    img.put_pixel(7, 4, eye_slit);
    img.put_pixel(8, 4, scale_light);
    img.put_pixel(9, 4, eye_slit);
    img.put_pixel(10, 4, eye_yellow);
    img.put_pixel(11, 4, scale_dark);

    // 口 (火を吐く)
    img.put_pixel(4, 5, scale_dark);
    img.put_pixel(5, 5, scale_mid);
    img.put_pixel(6, 5, scale_light);
    img.put_pixel(7, 5, scale_mid);
    img.put_pixel(8, 5, scale_mid);
    img.put_pixel(9, 5, scale_light);
    img.put_pixel(10, 5, scale_mid);
    img.put_pixel(11, 5, scale_dark);

    // 火炎ブレス (左側に吐く)
    img.put_pixel(1, 5, fire_yellow);
    img.put_pixel(2, 5, fire_orange);
    img.put_pixel(3, 5, fire_red);
    img.put_pixel(0, 4, fire_orange);
    img.put_pixel(1, 4, fire_red);
    img.put_pixel(2, 4, fire_orange);
    img.put_pixel(0, 6, fire_red);
    img.put_pixel(1, 6, fire_orange);
    img.put_pixel(2, 6, fire_red);

    // 首
    for x in 6..=9 {
        let color = if x == 6 || x == 9 { scale_dark } else { scale_mid };
        img.put_pixel(x, 6, color);
    }

    // 肩
    for x in 5..=10 {
        let color = if x == 5 || x == 10 {
            scale_dark
        } else if x == 7 || x == 8 {
            belly
        } else {
            scale_mid
        };
        img.put_pixel(x, 7, color);
    }

    // 翼 (左)
    img.put_pixel(2, 6, wing_dark);
    img.put_pixel(3, 6, wing_mid);
    img.put_pixel(4, 6, wing_dark);
    img.put_pixel(3, 7, wing_dark);
    img.put_pixel(4, 7, wing_mid);
    img.put_pixel(4, 8, wing_dark);

    // 翼 (右)
    img.put_pixel(11, 6, wing_dark);
    img.put_pixel(12, 6, wing_mid);
    img.put_pixel(13, 6, wing_dark);
    img.put_pixel(11, 7, wing_mid);
    img.put_pixel(12, 7, wing_dark);
    img.put_pixel(11, 8, wing_dark);

    // 胴体
    for y in 8..=10 {
        for x in 5..=10 {
            let color = if x == 5 || x == 10 {
                scale_dark
            } else if x == 7 || x == 8 {
                belly
            } else {
                scale_mid
            };
            img.put_pixel(x, y, color);
        }
    }

    // ウロコのハイライト
    img.put_pixel(6, 8, scale_bright);
    img.put_pixel(9, 9, scale_bright);
    img.put_pixel(6, 10, scale_bright);

    // 前脚
    img.put_pixel(4, 9, scale_mid);
    img.put_pixel(4, 10, scale_dark);
    img.put_pixel(3, 10, scale_dark);

    img.put_pixel(11, 9, scale_mid);
    img.put_pixel(11, 10, scale_dark);
    img.put_pixel(12, 10, scale_dark);

    // 下半身
    for x in 5..=10 {
        let color = if x == 5 || x == 10 { scale_dark } else { scale_mid };
        img.put_pixel(x, 11, color);
    }

    // 脚
    img.put_pixel(5, 12, scale_mid);
    img.put_pixel(6, 12, scale_dark);
    img.put_pixel(9, 12, scale_dark);
    img.put_pixel(10, 12, scale_mid);

    img.put_pixel(4, 13, scale_dark);
    img.put_pixel(5, 13, scale_mid);
    img.put_pixel(10, 13, scale_mid);
    img.put_pixel(11, 13, scale_dark);

    // 足
    img.put_pixel(3, 14, scale_dark);
    img.put_pixel(4, 14, scale_dark);
    img.put_pixel(5, 14, scale_dark);
    img.put_pixel(10, 14, scale_dark);
    img.put_pixel(11, 14, scale_dark);
    img.put_pixel(12, 14, scale_dark);

    // 尻尾 (右に伸びる)
    img.put_pixel(11, 11, scale_mid);
    img.put_pixel(12, 11, scale_dark);
    img.put_pixel(12, 12, scale_mid);
    img.put_pixel(13, 12, scale_dark);
    img.put_pixel(13, 13, scale_mid);
    img.put_pixel(14, 13, scale_dark);
    img.put_pixel(14, 14, scale_dark);

    save_image(&img, output_dir, "dragon.png");
}
