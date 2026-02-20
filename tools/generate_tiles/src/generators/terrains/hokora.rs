use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_noise, save_image, TILE_SIZE};

pub fn generate_hokora(output_dir: &Path) {
    let mut img = new_image();

    // --- パレット (8色) ---
    // 草地
    let grass_base = Rgba([90, 140, 80, 255]);
    let grass_dark = Rgba([70, 110, 60, 255]);

    // 鳥居（赤系）
    let torii_red = Rgba([190, 45, 40, 255]);
    let torii_dark = Rgba([140, 30, 30, 255]);

    // 祠本体（石灰色系）
    let stone = Rgba([175, 175, 170, 255]);
    let stone_dark = Rgba([130, 130, 125, 255]);

    // 屋根（暗めの灰色）
    let roof = Rgba([95, 95, 95, 255]);
    let roof_dark = Rgba([70, 70, 70, 255]);

    // --- 草地背景 ---
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let color = if pixel_noise(x, y, 80) < 0.2 {
                grass_dark
            } else {
                grass_base
            };
            img.put_pixel(x, y, color);
        }
    }

    // =========================================
    //  鳥居 (y=1..6)
    // =========================================

    // 笠木（上の横棒）: y=1, x=3..13
    for x in 3..13 {
        let color = if x == 3 || x == 12 {
            torii_dark
        } else {
            torii_red
        };
        img.put_pixel(x, 1, color);
    }

    // 島木（笠木の下）: y=2, x=4..12
    for x in 4..12 {
        let color = if x == 4 || x == 11 {
            torii_dark
        } else {
            torii_red
        };
        img.put_pixel(x, 2, color);
    }

    // 柱（左）: x=5, y=3..7
    for y in 3..7 {
        let color = if pixel_noise(5, y, 81) < 0.25 {
            torii_dark
        } else {
            torii_red
        };
        img.put_pixel(5, y, color);
    }

    // 柱（右）: x=10, y=3..7
    for y in 3..7 {
        let color = if pixel_noise(10, y, 82) < 0.25 {
            torii_dark
        } else {
            torii_red
        };
        img.put_pixel(10, y, color);
    }

    // 貫（柱中間の横棒）: y=4, x=5..11
    for x in 5..11 {
        img.put_pixel(x, 4, torii_dark);
    }

    // =========================================
    //  祠本体 (y=7..14)
    // =========================================

    // 屋根上段（広い庇）: y=7, x=4..12
    for x in 4..12 {
        let color = if x == 4 || x == 11 {
            roof_dark
        } else {
            roof
        };
        img.put_pixel(x, 7, color);
    }

    // 屋根中段: y=8, x=5..11
    for x in 5..11 {
        let color = if x == 5 || x == 10 {
            roof_dark
        } else {
            roof
        };
        img.put_pixel(x, 8, color);
    }

    // 屋根下段（軒先）: y=9, x=6..10
    for x in 6..10 {
        let color = if x == 6 {
            roof_dark
        } else {
            roof
        };
        img.put_pixel(x, 9, color);
    }

    // 壁: y=10..13, x=6..10
    for y in 10..13 {
        for x in 6..10 {
            let color = if x == 6 {
                stone_dark
            } else if x == 9 {
                stone
            } else if pixel_noise(x, y, 83) < 0.3 {
                stone_dark
            } else {
                stone
            };
            img.put_pixel(x, y, color);
        }
    }

    // 入口（暗い穴）: y=11..12, x=7..9
    let entrance = Rgba([45, 35, 35, 255]);
    for y in 11..13 {
        for x in 7..9 {
            img.put_pixel(x, y, entrance);
        }
    }

    // 石段: y=13, x=5..11
    for x in 5..11 {
        let color = if pixel_noise(x, 13, 84) < 0.4 {
            stone_dark
        } else {
            stone
        };
        img.put_pixel(x, 13, color);
    }

    // 石段（広め）: y=14, x=4..12
    for x in 4..12 {
        let color = if pixel_noise(x, 14, 85) < 0.4 {
            stone_dark
        } else {
            stone
        };
        img.put_pixel(x, 14, color);
    }

    save_image(&img, output_dir, "hokora.png");
}
