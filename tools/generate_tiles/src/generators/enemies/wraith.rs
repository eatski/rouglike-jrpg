use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_wraith(output_dir: &Path) {
    let mut img = new_image();

    let cloak_dark = Rgba([30, 15, 40, 255]);
    let cloak_mid = Rgba([50, 25, 65, 255]);
    let cloak_light = Rgba([70, 35, 85, 255]);
    let shadow = Rgba([20, 10, 25, 200]);
    let glow_blue = Rgba([120, 160, 255, 180]);
    let glow_bright = Rgba([180, 210, 255, 200]);
    let glow_core = Rgba([220, 240, 255, 220]);
    let wisp = Rgba([100, 130, 200, 120]);
    let wisp_faint = Rgba([80, 100, 180, 80]);

    // フード上部
    for x in 6..=9 {
        img.put_pixel(x, 1, cloak_dark);
    }
    for x in 5..=10 {
        let color = if x == 5 || x == 10 { cloak_dark } else { cloak_mid };
        img.put_pixel(x, 2, color);
    }

    // フード側面
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            cloak_dark
        } else if x == 5 || x == 10 {
            cloak_mid
        } else {
            cloak_light
        };
        img.put_pixel(x, 3, color);
    }

    // 顔 (目が光る暗い空洞)
    img.put_pixel(4, 4, cloak_dark);
    img.put_pixel(5, 4, cloak_mid);
    img.put_pixel(6, 4, glow_bright);
    img.put_pixel(7, 4, shadow);
    img.put_pixel(8, 4, shadow);
    img.put_pixel(9, 4, glow_bright);
    img.put_pixel(10, 4, cloak_mid);
    img.put_pixel(11, 4, cloak_dark);

    // 顔下部 (暗い空洞)
    img.put_pixel(4, 5, cloak_dark);
    img.put_pixel(5, 5, shadow);
    img.put_pixel(6, 5, glow_blue);
    img.put_pixel(7, 5, shadow);
    img.put_pixel(8, 5, shadow);
    img.put_pixel(9, 5, glow_blue);
    img.put_pixel(10, 5, shadow);
    img.put_pixel(11, 5, cloak_dark);

    // フード下端
    for x in 4..=11 {
        let color = if x == 4 || x == 11 { cloak_dark } else { cloak_mid };
        img.put_pixel(x, 6, color);
    }

    // 体 (ローブ)
    for y in 7..=10 {
        for x in 3..=12 {
            let color = if x == 3 || x == 12 {
                cloak_dark
            } else if x == 4 || x == 11 {
                cloak_mid
            } else if x == 7 || x == 8 {
                cloak_light
            } else {
                cloak_mid
            };
            img.put_pixel(x, y, color);
        }
    }

    // 腕のシルエット (左)
    img.put_pixel(2, 8, cloak_dark);
    img.put_pixel(2, 9, cloak_mid);
    img.put_pixel(1, 9, glow_blue);
    img.put_pixel(1, 10, wisp);

    // 腕のシルエット (右)
    img.put_pixel(13, 8, cloak_dark);
    img.put_pixel(13, 9, cloak_mid);
    img.put_pixel(14, 9, glow_blue);
    img.put_pixel(14, 10, wisp);

    // 下半身 (消えかけ/半透明)
    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            shadow
        } else if x == 4 || x == 11 {
            cloak_dark
        } else {
            cloak_mid
        };
        img.put_pixel(x, 11, color);
    }

    // 裾 (フェードアウト)
    for x in [4, 5, 7, 8, 10, 11] {
        img.put_pixel(x, 12, cloak_dark);
    }

    for x in [5, 8, 11] {
        img.put_pixel(x, 13, shadow);
    }

    // 霊体のオーラ/ウィスプ
    img.put_pixel(3, 6, wisp_faint);
    img.put_pixel(12, 6, wisp_faint);
    img.put_pixel(2, 7, wisp_faint);
    img.put_pixel(13, 7, wisp_faint);

    // 内部の青白い光
    img.put_pixel(7, 8, glow_blue);
    img.put_pixel(8, 8, glow_blue);
    img.put_pixel(7, 9, glow_core);
    img.put_pixel(8, 9, glow_core);
    img.put_pixel(7, 10, glow_blue);
    img.put_pixel(8, 10, glow_blue);

    save_image(&img, output_dir, "wraith.png");
}
