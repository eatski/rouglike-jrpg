use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_goblin(output_dir: &Path) {
    let mut img = new_image();

    let skin_green = Rgba([80, 120, 60, 255]);
    let skin_dark = Rgba([60, 90, 45, 255]);
    let skin_light = Rgba([100, 140, 80, 255]);
    let eye_red = Rgba([200, 50, 50, 255]);
    let eye_yellow = Rgba([220, 200, 80, 255]);
    let cloth_brown = Rgba([100, 70, 40, 255]);
    let cloth_dark = Rgba([70, 50, 30, 255]);
    let teeth = Rgba([230, 230, 220, 255]);
    let hair_dark = Rgba([40, 35, 30, 255]);

    // 頭部
    for x in 6..=9 {
        img.put_pixel(x, 2, hair_dark);
    }
    for x in 5..=10 {
        img.put_pixel(x, 3, if x == 5 || x == 10 { hair_dark } else { skin_green });
    }

    for x in 4..=11 {
        let color = if x == 4 || x == 11 { skin_dark } else { skin_green };
        img.put_pixel(x, 4, color);
    }

    // 目
    img.put_pixel(4, 5, skin_dark);
    img.put_pixel(5, 5, skin_green);
    img.put_pixel(6, 5, eye_yellow);
    img.put_pixel(7, 5, eye_red);
    img.put_pixel(8, 5, eye_red);
    img.put_pixel(9, 5, eye_yellow);
    img.put_pixel(10, 5, skin_green);
    img.put_pixel(11, 5, skin_dark);

    // 鼻・頬
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            skin_dark
        } else if x == 7 || x == 8 {
            skin_light
        } else {
            skin_green
        };
        img.put_pixel(x, 6, color);
    }

    // 口
    img.put_pixel(4, 7, skin_dark);
    img.put_pixel(5, 7, skin_dark);
    img.put_pixel(6, 7, teeth);
    img.put_pixel(7, 7, skin_dark);
    img.put_pixel(8, 7, skin_dark);
    img.put_pixel(9, 7, teeth);
    img.put_pixel(10, 7, skin_dark);
    img.put_pixel(11, 7, skin_dark);

    // 耳
    img.put_pixel(3, 4, skin_light);
    img.put_pixel(2, 5, skin_green);
    img.put_pixel(3, 5, skin_light);
    img.put_pixel(3, 6, skin_dark);

    img.put_pixel(12, 4, skin_light);
    img.put_pixel(12, 5, skin_light);
    img.put_pixel(13, 5, skin_green);
    img.put_pixel(12, 6, skin_dark);

    // 首
    for x in 6..=9 {
        img.put_pixel(x, 8, skin_green);
    }

    // 体
    for x in 4..=11 {
        let color = if x == 4 || x == 11 { cloth_dark } else { cloth_brown };
        img.put_pixel(x, 9, color);
    }

    for x in 5..=10 {
        img.put_pixel(x, 10, cloth_brown);
    }
    img.put_pixel(4, 10, cloth_dark);
    img.put_pixel(11, 10, cloth_dark);

    // 腕
    img.put_pixel(3, 10, skin_green);
    img.put_pixel(4, 11, skin_green);
    img.put_pixel(12, 10, skin_green);
    img.put_pixel(11, 11, skin_green);

    // 手
    img.put_pixel(3, 11, skin_dark);
    img.put_pixel(12, 11, skin_dark);

    // 下半身
    for x in 5..=10 {
        img.put_pixel(x, 11, cloth_dark);
    }

    // 脚
    img.put_pixel(5, 12, cloth_dark);
    img.put_pixel(6, 12, skin_green);
    img.put_pixel(9, 12, skin_green);
    img.put_pixel(10, 12, cloth_dark);

    // 足
    for x in [5, 6, 9, 10] {
        img.put_pixel(x, 13, skin_dark);
    }

    save_image(&img, output_dir, "goblin.png");
}
