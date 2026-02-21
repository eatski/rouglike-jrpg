use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_dark_lord(output_dir: &Path) {
    let mut img = new_image();

    let cape_dark = Rgba([40, 10, 50, 255]);
    let cape_mid = Rgba([60, 20, 70, 255]);
    let cape_light = Rgba([80, 30, 90, 255]);
    let armor_dark = Rgba([30, 30, 35, 255]);
    let armor_mid = Rgba([50, 45, 55, 255]);
    let armor_light = Rgba([70, 65, 80, 255]);
    let armor_highlight = Rgba([100, 90, 110, 255]);
    let eye_red = Rgba([255, 40, 40, 255]);
    let eye_glow = Rgba([255, 80, 60, 255]);
    let crown_gold = Rgba([200, 170, 50, 255]);
    let crown_dark = Rgba([160, 130, 30, 255]);
    let skin = Rgba([180, 160, 170, 255]);

    // 王冠
    for x in [6, 8, 10] {
        img.put_pixel(x, 0, crown_gold);
    }
    for x in 5..=11 {
        let color = if x == 5 || x == 11 { crown_dark } else { crown_gold };
        img.put_pixel(x, 1, color);
    }

    // 頭部（兜）
    for x in 5..=10 {
        let color = if x == 5 || x == 10 { armor_dark } else { armor_mid };
        img.put_pixel(x, 2, color);
    }

    // 顔
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            armor_dark
        } else if x == 6 || x == 9 {
            eye_red
        } else if x == 7 || x == 8 {
            skin
        } else {
            armor_mid
        };
        img.put_pixel(x, 3, color);
    }

    // 顎
    for x in 5..=10 {
        let color = if x == 5 || x == 10 { armor_dark } else { skin };
        img.put_pixel(x, 4, color);
    }

    // 肩甲（広い肩）
    for x in 2..=13 {
        let color = if x <= 3 || x >= 12 {
            cape_dark
        } else if x == 4 || x == 11 {
            armor_light
        } else {
            armor_mid
        };
        img.put_pixel(x, 5, color);
    }

    // トゲ肩パッド
    img.put_pixel(1, 4, cape_mid);
    img.put_pixel(2, 4, armor_highlight);
    img.put_pixel(13, 4, armor_highlight);
    img.put_pixel(14, 4, cape_mid);

    // 胸部
    for y in 6..=8 {
        for x in 3..=12 {
            let color = if x == 3 || x == 12 {
                cape_dark
            } else if x == 4 || x == 11 {
                cape_mid
            } else if x == 7 || x == 8 {
                armor_highlight
            } else {
                armor_mid
            };
            img.put_pixel(x, y, color);
        }
    }

    // 腰帯
    for x in 4..=11 {
        let color = if x == 7 || x == 8 { crown_gold } else { armor_dark };
        img.put_pixel(x, 9, color);
    }

    // マント下部（広がる）
    for y in 10..=13 {
        let spread = y - 10;
        let left = 3i32 - spread as i32;
        let right = 12i32 + spread as i32;
        for x in left.max(0)..=right.min(15) {
            let x = x as u32;
            let is_edge = x as i32 == left || x as i32 == right;
            let color = if is_edge {
                cape_dark
            } else if x <= 4 || x >= 11 {
                cape_mid
            } else {
                cape_light
            };
            img.put_pixel(x, y, color);
        }
    }

    // 足
    for x in 5..=6 {
        img.put_pixel(x, 14, armor_dark);
    }
    for x in 9..=10 {
        img.put_pixel(x, 14, armor_dark);
    }

    // 目の光
    img.put_pixel(6, 3, eye_glow);
    img.put_pixel(9, 3, eye_glow);

    save_image(&img, output_dir, "dark_lord.png");
}
