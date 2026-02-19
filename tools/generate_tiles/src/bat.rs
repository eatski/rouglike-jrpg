use image::Rgba;
use std::path::Path;

use crate::common::{new_image, save_image};

pub fn generate_bat(output_dir: &Path) {
    let mut img = new_image();

    let body_dark = Rgba([40, 20, 60, 255]);
    let body_mid = Rgba([60, 30, 90, 255]);
    let wing_dark = Rgba([50, 30, 70, 255]);
    let wing_mid = Rgba([70, 40, 100, 255]);
    let wing_light = Rgba([90, 50, 120, 255]);
    let eye_red = Rgba([200, 50, 50, 255]);

    // 体
    for y in 7..=10 {
        for x in 7..=8 {
            img.put_pixel(x, y, body_mid);
        }
    }
    img.put_pixel(7, 10, body_dark);
    img.put_pixel(8, 10, body_dark);

    // 頭部
    img.put_pixel(7, 6, body_mid);
    img.put_pixel(8, 6, body_mid);
    img.put_pixel(7, 5, body_dark);
    img.put_pixel(8, 5, body_dark);

    // 目
    img.put_pixel(6, 6, eye_red);
    img.put_pixel(9, 6, eye_red);

    // 耳
    img.put_pixel(6, 4, body_dark);
    img.put_pixel(9, 4, body_dark);
    img.put_pixel(7, 5, body_mid);
    img.put_pixel(8, 5, body_mid);

    // 左翼
    for x in 2..=5 {
        img.put_pixel(x, 8, wing_dark);
    }
    img.put_pixel(1, 9, wing_light);
    img.put_pixel(2, 9, wing_mid);
    img.put_pixel(3, 9, wing_mid);
    img.put_pixel(4, 9, wing_dark);

    img.put_pixel(1, 10, wing_mid);
    img.put_pixel(2, 10, wing_mid);
    img.put_pixel(3, 10, wing_dark);

    img.put_pixel(2, 11, wing_mid);
    img.put_pixel(3, 11, wing_dark);

    img.put_pixel(3, 12, wing_dark);
    img.put_pixel(4, 12, wing_dark);

    img.put_pixel(2, 7, wing_dark);
    img.put_pixel(3, 7, wing_mid);
    img.put_pixel(4, 7, wing_light);
    img.put_pixel(5, 7, wing_mid);

    // 右翼
    for x in 10..=13 {
        img.put_pixel(x, 8, wing_dark);
    }
    img.put_pixel(11, 9, wing_dark);
    img.put_pixel(12, 9, wing_mid);
    img.put_pixel(13, 9, wing_mid);
    img.put_pixel(14, 9, wing_light);

    img.put_pixel(12, 10, wing_dark);
    img.put_pixel(13, 10, wing_mid);
    img.put_pixel(14, 10, wing_mid);

    img.put_pixel(12, 11, wing_dark);
    img.put_pixel(13, 11, wing_mid);

    img.put_pixel(11, 12, wing_dark);
    img.put_pixel(12, 12, wing_dark);

    img.put_pixel(10, 7, wing_mid);
    img.put_pixel(11, 7, wing_light);
    img.put_pixel(12, 7, wing_mid);
    img.put_pixel(13, 7, wing_dark);

    // 足
    img.put_pixel(7, 11, body_dark);
    img.put_pixel(8, 11, body_dark);

    save_image(&img, output_dir, "bat.png");
}
