use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_lizardman(output_dir: &Path) {
    let mut img = new_image();

    let scale_dark = Rgba([30, 80, 60, 255]);
    let scale_mid = Rgba([50, 120, 90, 255]);
    let scale_light = Rgba([70, 150, 110, 255]);
    let scale_bright = Rgba([90, 170, 130, 255]);
    let belly = Rgba([140, 170, 120, 255]);
    let eye_yellow = Rgba([240, 220, 60, 255]);
    let eye_slit = Rgba([20, 20, 10, 255]);
    let mouth = Rgba([180, 60, 60, 255]);
    let claw = Rgba([200, 190, 170, 255]);

    // 頭部 (爬虫類風の長い頭)
    for x in 6..=10 {
        img.put_pixel(x, 1, scale_dark);
    }

    for x in 5..=11 {
        let color = if x == 5 || x == 11 { scale_dark } else { scale_mid };
        img.put_pixel(x, 2, color);
    }

    // 目
    img.put_pixel(5, 3, scale_dark);
    img.put_pixel(6, 3, eye_yellow);
    img.put_pixel(7, 3, eye_slit);
    img.put_pixel(8, 3, scale_light);
    img.put_pixel(9, 3, eye_slit);
    img.put_pixel(10, 3, eye_yellow);
    img.put_pixel(11, 3, scale_dark);

    // 鼻・口
    img.put_pixel(4, 4, scale_dark);
    img.put_pixel(5, 4, scale_mid);
    img.put_pixel(6, 4, scale_light);
    img.put_pixel(7, 4, scale_mid);
    img.put_pixel(8, 4, scale_mid);
    img.put_pixel(9, 4, scale_light);
    img.put_pixel(10, 4, scale_mid);
    img.put_pixel(11, 4, scale_dark);

    // 口
    img.put_pixel(5, 5, scale_dark);
    img.put_pixel(6, 5, mouth);
    img.put_pixel(7, 5, claw); // 歯
    img.put_pixel(8, 5, mouth);
    img.put_pixel(9, 5, claw); // 歯
    img.put_pixel(10, 5, scale_dark);

    // 首
    for x in 6..=9 {
        let color = if x == 6 || x == 9 { scale_dark } else { scale_mid };
        img.put_pixel(x, 6, color);
    }

    // 肩
    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            scale_dark
        } else if x == 4 || x == 11 {
            scale_mid
        } else if x == 7 || x == 8 {
            belly
        } else {
            scale_light
        };
        img.put_pixel(x, 7, color);
    }

    // 胴体
    for y in 8..=10 {
        for x in 4..=11 {
            let color = if x == 4 || x == 11 {
                scale_dark
            } else if x == 5 || x == 10 {
                scale_mid
            } else if x == 7 || x == 8 {
                belly
            } else {
                scale_light
            };
            img.put_pixel(x, y, color);
        }
    }

    // 腕 (左)
    img.put_pixel(2, 7, scale_mid);
    img.put_pixel(2, 8, scale_dark);
    img.put_pixel(3, 8, scale_mid);
    img.put_pixel(3, 9, scale_dark);
    img.put_pixel(2, 9, claw);

    // 腕 (右)
    img.put_pixel(13, 7, scale_mid);
    img.put_pixel(13, 8, scale_dark);
    img.put_pixel(12, 8, scale_mid);
    img.put_pixel(12, 9, scale_dark);
    img.put_pixel(13, 9, claw);

    // 下半身
    for x in 5..=10 {
        let color = if x == 5 || x == 10 {
            scale_dark
        } else if x == 7 || x == 8 {
            belly
        } else {
            scale_mid
        };
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

    // 足 (爪付き)
    img.put_pixel(3, 14, claw);
    img.put_pixel(4, 14, scale_dark);
    img.put_pixel(5, 14, scale_dark);
    img.put_pixel(10, 14, scale_dark);
    img.put_pixel(11, 14, scale_dark);
    img.put_pixel(12, 14, claw);

    // 尻尾 (右に伸びる)
    img.put_pixel(11, 11, scale_mid);
    img.put_pixel(12, 11, scale_dark);
    img.put_pixel(12, 12, scale_mid);
    img.put_pixel(13, 12, scale_dark);
    img.put_pixel(13, 13, scale_mid);
    img.put_pixel(14, 13, scale_dark);

    // ウロコのハイライト
    img.put_pixel(6, 8, scale_bright);
    img.put_pixel(9, 9, scale_bright);
    img.put_pixel(6, 10, scale_bright);

    save_image(&img, output_dir, "lizardman.png");
}
