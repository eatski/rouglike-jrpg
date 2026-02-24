use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_skeleton(output_dir: &Path) {
    let mut img = new_image();

    let bone_white = Rgba([230, 225, 210, 255]);
    let bone_light = Rgba([200, 195, 180, 255]);
    let bone_mid = Rgba([170, 165, 155, 255]);
    let bone_dark = Rgba([130, 125, 120, 255]);
    let eye_socket = Rgba([30, 10, 15, 255]);
    let sword_blade = Rgba([180, 190, 200, 255]);
    let sword_hilt = Rgba([120, 80, 40, 255]);

    // 頭蓋骨
    for x in 6..=9 {
        img.put_pixel(x, 1, bone_dark);
    }
    for x in 5..=10 {
        let color = if x == 5 || x == 10 { bone_dark } else { bone_white };
        img.put_pixel(x, 2, color);
    }

    // 目
    img.put_pixel(5, 3, bone_dark);
    img.put_pixel(6, 3, eye_socket);
    img.put_pixel(7, 3, bone_light);
    img.put_pixel(8, 3, eye_socket);
    img.put_pixel(9, 3, bone_light);
    img.put_pixel(10, 3, bone_dark);

    // 鼻
    img.put_pixel(5, 4, bone_dark);
    img.put_pixel(6, 4, bone_white);
    img.put_pixel(7, 4, bone_mid);
    img.put_pixel(8, 4, bone_mid);
    img.put_pixel(9, 4, bone_white);
    img.put_pixel(10, 4, bone_dark);

    // 顎・歯
    img.put_pixel(6, 5, bone_dark);
    img.put_pixel(7, 5, bone_white);
    img.put_pixel(8, 5, bone_white);
    img.put_pixel(9, 5, bone_dark);

    // 首 (背骨)
    img.put_pixel(7, 6, bone_mid);
    img.put_pixel(8, 6, bone_mid);

    // 肩・鎖骨
    for x in 4..=11 {
        let color = if x == 7 || x == 8 {
            bone_light
        } else if x == 4 || x == 11 {
            bone_dark
        } else {
            bone_mid
        };
        img.put_pixel(x, 7, color);
    }

    // リブケージ (肋骨)
    img.put_pixel(5, 8, bone_mid);
    img.put_pixel(6, 8, bone_light);
    img.put_pixel(7, 8, bone_dark);
    img.put_pixel(8, 8, bone_dark);
    img.put_pixel(9, 8, bone_light);
    img.put_pixel(10, 8, bone_mid);

    img.put_pixel(5, 9, bone_dark);
    img.put_pixel(6, 9, bone_light);
    img.put_pixel(7, 9, bone_mid);
    img.put_pixel(8, 9, bone_mid);
    img.put_pixel(9, 9, bone_light);
    img.put_pixel(10, 9, bone_dark);

    img.put_pixel(6, 10, bone_mid);
    img.put_pixel(7, 10, bone_light);
    img.put_pixel(8, 10, bone_light);
    img.put_pixel(9, 10, bone_mid);

    // 背骨 (腰)
    img.put_pixel(7, 11, bone_mid);
    img.put_pixel(8, 11, bone_mid);

    // 骨盤
    img.put_pixel(6, 12, bone_dark);
    img.put_pixel(7, 12, bone_light);
    img.put_pixel(8, 12, bone_light);
    img.put_pixel(9, 12, bone_dark);

    // 脚
    img.put_pixel(6, 13, bone_mid);
    img.put_pixel(9, 13, bone_mid);
    img.put_pixel(5, 14, bone_dark);
    img.put_pixel(6, 14, bone_mid);
    img.put_pixel(9, 14, bone_mid);
    img.put_pixel(10, 14, bone_dark);

    // 左腕
    img.put_pixel(4, 8, bone_mid);
    img.put_pixel(3, 9, bone_mid);
    img.put_pixel(3, 10, bone_dark);

    // 右腕 (剣を持つ)
    img.put_pixel(11, 8, bone_mid);
    img.put_pixel(12, 9, bone_mid);
    img.put_pixel(12, 10, bone_dark);

    // 剣 (右手に持つ)
    img.put_pixel(13, 10, sword_hilt);
    img.put_pixel(13, 9, sword_blade);
    img.put_pixel(13, 8, sword_blade);
    img.put_pixel(13, 7, sword_blade);
    img.put_pixel(13, 6, sword_blade);
    img.put_pixel(13, 5, sword_blade);
    img.put_pixel(14, 10, sword_hilt);

    save_image(&img, output_dir, "skeleton.png");
}
