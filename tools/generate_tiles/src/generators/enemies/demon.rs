use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_demon(output_dir: &Path) {
    let mut img = new_image();

    let skin_dark = Rgba([100, 20, 50, 255]);
    let skin_mid = Rgba([140, 40, 70, 255]);
    let skin_light = Rgba([170, 60, 90, 255]);
    let horn_dark = Rgba([60, 30, 20, 255]);
    let horn_light = Rgba([100, 60, 40, 255]);
    let wing_dark = Rgba([80, 15, 40, 255]);
    let wing_mid = Rgba([110, 30, 55, 255]);
    let eye_yellow = Rgba([255, 220, 50, 255]);
    let eye_red = Rgba([255, 50, 30, 255]);
    let mouth = Rgba([200, 30, 30, 255]);
    let teeth = Rgba([230, 230, 210, 255]);

    // 角 (左)
    img.put_pixel(4, 0, horn_dark);
    img.put_pixel(5, 0, horn_light);
    img.put_pixel(5, 1, horn_dark);
    img.put_pixel(6, 1, horn_light);

    // 角 (右)
    img.put_pixel(10, 0, horn_light);
    img.put_pixel(11, 0, horn_dark);
    img.put_pixel(9, 1, horn_light);
    img.put_pixel(10, 1, horn_dark);

    // 頭部
    for x in 6..=9 {
        img.put_pixel(x, 2, skin_dark);
    }
    for x in 5..=10 {
        let color = if x == 5 || x == 10 { skin_dark } else { skin_mid };
        img.put_pixel(x, 3, color);
    }

    // 目
    img.put_pixel(5, 4, skin_dark);
    img.put_pixel(6, 4, eye_yellow);
    img.put_pixel(7, 4, eye_red);
    img.put_pixel(8, 4, eye_red);
    img.put_pixel(9, 4, eye_yellow);
    img.put_pixel(10, 4, skin_dark);

    // 口 (牙)
    img.put_pixel(5, 5, skin_dark);
    img.put_pixel(6, 5, teeth);
    img.put_pixel(7, 5, mouth);
    img.put_pixel(8, 5, mouth);
    img.put_pixel(9, 5, teeth);
    img.put_pixel(10, 5, skin_dark);

    // 首
    img.put_pixel(7, 6, skin_mid);
    img.put_pixel(8, 6, skin_mid);

    // 肩・胴体上部
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            skin_dark
        } else if x == 5 || x == 10 {
            skin_mid
        } else {
            skin_light
        };
        img.put_pixel(x, 7, color);
    }

    // 翼 (左)
    img.put_pixel(1, 5, wing_dark);
    img.put_pixel(2, 5, wing_mid);
    img.put_pixel(0, 6, wing_dark);
    img.put_pixel(1, 6, wing_mid);
    img.put_pixel(2, 6, wing_dark);
    img.put_pixel(3, 6, wing_mid);
    img.put_pixel(0, 7, wing_mid);
    img.put_pixel(1, 7, wing_dark);
    img.put_pixel(2, 7, wing_mid);
    img.put_pixel(3, 7, wing_dark);
    img.put_pixel(1, 8, wing_dark);
    img.put_pixel(2, 8, wing_mid);
    img.put_pixel(3, 8, wing_dark);
    img.put_pixel(2, 9, wing_dark);

    // 翼 (右)
    img.put_pixel(13, 5, wing_mid);
    img.put_pixel(14, 5, wing_dark);
    img.put_pixel(12, 6, wing_mid);
    img.put_pixel(13, 6, wing_dark);
    img.put_pixel(14, 6, wing_mid);
    img.put_pixel(15, 6, wing_dark);
    img.put_pixel(12, 7, wing_dark);
    img.put_pixel(13, 7, wing_mid);
    img.put_pixel(14, 7, wing_dark);
    img.put_pixel(15, 7, wing_mid);
    img.put_pixel(12, 8, wing_dark);
    img.put_pixel(13, 8, wing_mid);
    img.put_pixel(14, 8, wing_dark);
    img.put_pixel(13, 9, wing_dark);

    // 胴体
    for y in 8..=10 {
        for x in 4..=11 {
            let color = if x == 4 || x == 11 {
                skin_dark
            } else if x == 5 || x == 10 {
                skin_mid
            } else {
                skin_light
            };
            img.put_pixel(x, y, color);
        }
    }

    // 腹部のディテール
    img.put_pixel(7, 9, skin_mid);
    img.put_pixel(8, 9, skin_mid);

    // 下半身
    for x in 5..=10 {
        let color = if x == 5 || x == 10 { skin_dark } else { skin_mid };
        img.put_pixel(x, 11, color);
    }

    // 脚
    img.put_pixel(5, 12, skin_mid);
    img.put_pixel(6, 12, skin_dark);
    img.put_pixel(9, 12, skin_dark);
    img.put_pixel(10, 12, skin_mid);

    img.put_pixel(5, 13, skin_dark);
    img.put_pixel(6, 13, skin_mid);
    img.put_pixel(9, 13, skin_mid);
    img.put_pixel(10, 13, skin_dark);

    // 足 (蹄風)
    img.put_pixel(4, 14, skin_dark);
    img.put_pixel(5, 14, skin_dark);
    img.put_pixel(10, 14, skin_dark);
    img.put_pixel(11, 14, skin_dark);

    save_image(&img, output_dir, "demon.png");
}
