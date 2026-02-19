use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, save_image};

pub fn generate_player(output_dir: &Path) {
    let mut img = new_image();

    let skin = Rgba([255, 210, 170, 255]);
    let hair = Rgba([220, 180, 80, 255]);
    let hair_dark = Rgba([180, 140, 60, 255]);
    let eye = Rgba([40, 60, 120, 255]);
    let armor = Rgba([180, 180, 200, 255]);
    let armor_light = Rgba([220, 220, 240, 255]);
    let armor_dark = Rgba([120, 120, 140, 255]);
    let cape_red = Rgba([180, 50, 50, 255]);
    let cape_dark = Rgba([140, 30, 30, 255]);
    let gold = Rgba([255, 215, 0, 255]);
    let sword = Rgba([200, 210, 220, 255]);
    let sword_light = Rgba([240, 245, 255, 255]);
    let boots = Rgba([80, 60, 40, 255]);

    // 剣
    img.put_pixel(13, 2, sword_light);
    img.put_pixel(13, 3, sword);
    img.put_pixel(13, 4, sword);
    img.put_pixel(13, 5, sword);
    img.put_pixel(13, 6, sword);
    img.put_pixel(13, 7, gold);
    img.put_pixel(12, 7, gold);
    img.put_pixel(14, 7, gold);
    img.put_pixel(13, 8, Rgba([100, 70, 40, 255]));

    // 髪
    img.put_pixel(7, 0, hair);
    img.put_pixel(8, 0, hair);
    for x in 5..=10 {
        img.put_pixel(x, 1, if x == 6 || x == 9 { hair } else { hair_dark });
    }
    img.put_pixel(6, 0, hair);
    img.put_pixel(9, 0, hair);
    for x in 4..=11 {
        img.put_pixel(x, 2, if x == 5 || x == 10 { hair } else { hair_dark });
    }
    for x in 4..=11 {
        img.put_pixel(x, 3, hair_dark);
    }
    for x in 5..=10 {
        img.put_pixel(x, 4, hair);
    }

    // 顔
    for x in 5..=10 {
        img.put_pixel(x, 5, skin);
    }
    img.put_pixel(5, 6, skin);
    img.put_pixel(6, 6, eye);
    img.put_pixel(7, 6, skin);
    img.put_pixel(8, 6, skin);
    img.put_pixel(9, 6, eye);
    img.put_pixel(10, 6, skin);
    for x in 5..=10 {
        img.put_pixel(x, 7, skin);
    }

    // 鎧（上半身）
    img.put_pixel(3, 8, armor_dark);
    img.put_pixel(4, 8, armor);
    img.put_pixel(11, 8, armor);
    img.put_pixel(12, 8, armor_light);

    for x in 5..=10 {
        img.put_pixel(x, 8, if x < 8 { armor_dark } else { armor });
    }
    for x in 4..=11 {
        let c = if x == 7 || x == 8 { gold } else if x < 7 { armor_dark } else { armor };
        img.put_pixel(x, 9, c);
    }
    for x in 5..=10 {
        let c = if x == 7 || x == 8 { gold } else if x < 7 { armor_dark } else { armor_light };
        img.put_pixel(x, 10, c);
    }
    for x in 5..=10 {
        img.put_pixel(x, 11, if x < 7 { armor_dark } else { armor });
    }

    // マント
    img.put_pixel(2, 9, cape_dark);
    img.put_pixel(2, 10, cape_red);
    img.put_pixel(2, 11, cape_red);
    img.put_pixel(3, 10, cape_dark);
    img.put_pixel(3, 11, cape_red);

    // 腕
    img.put_pixel(3, 9, armor);
    img.put_pixel(4, 10, skin);
    img.put_pixel(12, 9, armor_light);
    img.put_pixel(11, 10, skin);

    // 腰ベルト
    for x in 5..=10 {
        img.put_pixel(x, 12, if x == 7 || x == 8 { gold } else { armor_dark });
    }

    // 脚
    img.put_pixel(5, 13, armor_dark);
    img.put_pixel(6, 13, armor);
    img.put_pixel(9, 13, armor);
    img.put_pixel(10, 13, armor_light);

    // マント下部
    img.put_pixel(2, 12, cape_red);
    img.put_pixel(2, 13, cape_dark);
    img.put_pixel(3, 12, cape_dark);
    img.put_pixel(3, 13, cape_red);

    // ブーツ
    img.put_pixel(5, 14, boots);
    img.put_pixel(6, 14, boots);
    img.put_pixel(9, 14, boots);
    img.put_pixel(10, 14, boots);

    save_image(&img, output_dir, "player.png");
}
