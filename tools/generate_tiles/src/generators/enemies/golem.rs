use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_golem(output_dir: &Path) {
    let mut img = new_image();

    let rock_dark = Rgba([70, 65, 60, 255]);
    let rock_mid = Rgba([110, 105, 95, 255]);
    let rock_light = Rgba([145, 140, 130, 255]);
    let rock_bright = Rgba([170, 165, 155, 255]);
    let eye_glow = Rgba([255, 200, 50, 255]);
    let eye_core = Rgba([255, 255, 150, 255]);
    let crack = Rgba([50, 45, 40, 255]);
    let moss = Rgba([60, 90, 50, 255]);

    // 頭部 (四角い岩)
    for x in 5..=10 {
        img.put_pixel(x, 1, rock_dark);
    }
    for x in 4..=11 {
        let color = if x == 4 || x == 11 { rock_dark } else { rock_mid };
        img.put_pixel(x, 2, color);
    }

    // 目 (光る)
    img.put_pixel(4, 3, rock_dark);
    img.put_pixel(5, 3, rock_mid);
    img.put_pixel(6, 3, eye_glow);
    img.put_pixel(7, 3, eye_core);
    img.put_pixel(8, 3, rock_light);
    img.put_pixel(9, 3, eye_core);
    img.put_pixel(10, 3, eye_glow);
    img.put_pixel(11, 3, rock_dark);

    // 顔下部
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            rock_dark
        } else if x == 7 || x == 8 {
            crack
        } else {
            rock_mid
        };
        img.put_pixel(x, 4, color);
    }
    for x in 5..=10 {
        img.put_pixel(x, 5, rock_dark);
    }

    // 肩 (非常に広い)
    for x in 1..=14 {
        let color = if x <= 2 || x >= 13 {
            rock_dark
        } else if x == 3 || x == 12 {
            rock_mid
        } else if x == 7 || x == 8 {
            rock_light
        } else {
            rock_mid
        };
        img.put_pixel(x, 6, color);
    }

    // 胴体 (大きくて四角い)
    for y in 7..=11 {
        for x in 2..=13 {
            let color = if x == 2 || x == 13 {
                rock_dark
            } else if x == 3 || x == 12 {
                rock_mid
            } else if x == 7 || x == 8 {
                rock_light
            } else if x == 6 || x == 9 {
                rock_bright
            } else {
                rock_mid
            };
            img.put_pixel(x, y, color);
        }
    }

    // ヒビ (体に亀裂)
    img.put_pixel(5, 8, crack);
    img.put_pixel(6, 9, crack);
    img.put_pixel(5, 10, crack);
    img.put_pixel(10, 7, crack);
    img.put_pixel(10, 8, crack);
    img.put_pixel(11, 9, crack);

    // 苔
    img.put_pixel(3, 7, moss);
    img.put_pixel(4, 6, moss);
    img.put_pixel(12, 8, moss);

    // 腕 (太い)
    img.put_pixel(0, 7, rock_mid);
    img.put_pixel(1, 7, rock_mid);
    img.put_pixel(0, 8, rock_dark);
    img.put_pixel(1, 8, rock_mid);
    img.put_pixel(0, 9, rock_mid);
    img.put_pixel(1, 9, rock_dark);
    img.put_pixel(0, 10, rock_dark);
    img.put_pixel(1, 10, rock_mid);

    img.put_pixel(14, 7, rock_mid);
    img.put_pixel(15, 7, rock_mid);
    img.put_pixel(14, 8, rock_mid);
    img.put_pixel(15, 8, rock_dark);
    img.put_pixel(14, 9, rock_dark);
    img.put_pixel(15, 9, rock_mid);
    img.put_pixel(14, 10, rock_mid);
    img.put_pixel(15, 10, rock_dark);

    // 拳
    img.put_pixel(0, 11, rock_dark);
    img.put_pixel(1, 11, rock_dark);
    img.put_pixel(14, 11, rock_dark);
    img.put_pixel(15, 11, rock_dark);

    // 脚 (太くて短い)
    for x in 3..=6 {
        let color = if x == 3 || x == 6 { rock_dark } else { rock_mid };
        img.put_pixel(x, 12, color);
        img.put_pixel(x, 13, color);
    }
    for x in 9..=12 {
        let color = if x == 9 || x == 12 { rock_dark } else { rock_mid };
        img.put_pixel(x, 12, color);
        img.put_pixel(x, 13, color);
    }

    // 足
    for x in 2..=6 {
        img.put_pixel(x, 14, rock_dark);
    }
    for x in 9..=13 {
        img.put_pixel(x, 14, rock_dark);
    }

    save_image(&img, output_dir, "golem.png");
}
