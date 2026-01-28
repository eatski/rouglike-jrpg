use image::{ImageBuffer, Rgba, RgbaImage};
use rand::Rng;
use std::fs;
use std::path::Path;

const TILE_SIZE: u32 = 16;

fn main() {
    let tiles_dir = Path::new("assets/tiles");
    let chars_dir = Path::new("assets/characters");
    fs::create_dir_all(tiles_dir).expect("Failed to create tiles directory");
    fs::create_dir_all(chars_dir).expect("Failed to create characters directory");

    // 各地形タイルを生成
    generate_sea(tiles_dir);
    generate_plains(tiles_dir);
    generate_forest(tiles_dir);
    generate_mountain(tiles_dir);

    // キャラクターを生成
    generate_player(chars_dir);

    println!("Assets generated in assets/");
}

fn generate_sea(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);
    let mut rng = rand::thread_rng();

    // ベースカラー（深い青）
    let base = Rgba([40, 80, 120, 255]);
    let light = Rgba([60, 120, 180, 255]);
    let highlight = Rgba([100, 160, 220, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            // 波模様（横方向のライン）
            let wave = ((y as f32 / 4.0).sin() * 2.0 + (x as f32 / 3.0).cos()) as i32;
            let color = if (y as i32 + wave) % 6 < 2 {
                light
            } else if rng.gen_bool(0.05) {
                highlight // 泡・きらめき
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    img.save(output_dir.join("sea.png")).expect("Failed to save sea.png");
    println!("Generated: sea.png");
}

fn generate_plains(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);
    let mut rng = rand::thread_rng();

    // 草原カラーパレット
    let base = Rgba([120, 180, 100, 255]);
    let dark = Rgba([80, 140, 70, 255]);
    let light = Rgba([150, 210, 120, 255]);
    let flower = Rgba([220, 180, 80, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r: f32 = rng.r#gen();
            let color = if r < 0.15 {
                dark // 暗い草
            } else if r < 0.25 {
                light // 明るい草
            } else if r < 0.28 {
                flower // 小さな花
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    img.save(output_dir.join("plains.png")).expect("Failed to save plains.png");
    println!("Generated: plains.png");
}

fn generate_forest(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);
    let mut rng = rand::thread_rng();

    // 森林カラーパレット
    let base = Rgba([25, 100, 50, 255]);
    let dark = Rgba([15, 70, 35, 255]);
    let light = Rgba([45, 130, 70, 255]);
    let trunk = Rgba([80, 50, 30, 255]);

    // まずベースで埋める
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, base);
        }
    }

    // 木のパターンを配置（2-3本）
    let tree_positions = [(4, 8), (11, 6), (7, 13)];
    for (tx, ty) in tree_positions {
        // 幹
        if ty < TILE_SIZE && tx < TILE_SIZE {
            img.put_pixel(tx, ty, trunk);
            if ty + 1 < TILE_SIZE {
                img.put_pixel(tx, ty + 1, trunk);
            }
        }
        // 葉（幹の上に三角形状）
        for dy in 0..4 {
            let leaf_y = ty.saturating_sub(dy + 1);
            let width = (dy + 1).min(3);
            for dx in 0..=width {
                let lx1 = tx.saturating_sub(dx / 2);
                let lx2 = (tx + dx / 2).min(TILE_SIZE - 1);
                if leaf_y < TILE_SIZE {
                    if lx1 < TILE_SIZE {
                        let c = if rng.gen_bool(0.3) { light } else { dark };
                        img.put_pixel(lx1, leaf_y, c);
                    }
                    if lx2 < TILE_SIZE && lx2 != lx1 {
                        let c = if rng.gen_bool(0.3) { light } else { dark };
                        img.put_pixel(lx2, leaf_y, c);
                    }
                }
            }
        }
    }

    // ランダムな葉のノイズ
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            if img.get_pixel(x, y) == &base && rng.gen_bool(0.2) {
                let c = if rng.gen_bool(0.5) { dark } else { light };
                img.put_pixel(x, y, c);
            }
        }
    }

    img.save(output_dir.join("forest.png")).expect("Failed to save forest.png");
    println!("Generated: forest.png");
}

fn generate_mountain(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);
    let mut rng = rand::thread_rng();

    // 山岳カラーパレット（岩っぽいグレー系）
    let bg = Rgba([60, 80, 60, 255]);        // 背景（暗い緑がかったグレー）
    let rock_dark = Rgba([70, 70, 80, 255]); // 暗い岩
    let rock_mid = Rgba([100, 100, 110, 255]); // 中間の岩
    let rock_light = Rgba([140, 140, 150, 255]); // 明るい岩（ハイライト）
    let snow = Rgba([220, 230, 240, 255]);   // 雪
    let snow_shadow = Rgba([180, 190, 210, 255]); // 雪の影

    // 背景で埋める
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, bg);
        }
    }

    // メインの山（中央、大きめ）
    draw_mountain_peak(&mut img, &mut rng, 8, 2, 12, rock_dark, rock_mid, rock_light, snow, snow_shadow);

    // サブの山（左奥、小さめ）
    draw_mountain_peak(&mut img, &mut rng, 3, 5, 6, rock_dark, rock_mid, rock_light, snow, snow_shadow);

    // サブの山（右奥、小さめ）
    draw_mountain_peak(&mut img, &mut rng, 13, 6, 5, rock_dark, rock_mid, rock_light, snow, snow_shadow);

    img.save(output_dir.join("mountain.png")).expect("Failed to save mountain.png");
    println!("Generated: mountain.png");
}

fn draw_mountain_peak(
    img: &mut RgbaImage,
    rng: &mut impl rand::Rng,
    peak_x: u32,
    peak_y: u32,
    height: u32,
    rock_dark: Rgba<u8>,
    rock_mid: Rgba<u8>,
    rock_light: Rgba<u8>,
    snow: Rgba<u8>,
    snow_shadow: Rgba<u8>,
) {
    let snow_line = height / 4; // 上から1/4が雪

    for dy in 0..height {
        let y = peak_y + dy;
        if y >= TILE_SIZE {
            break;
        }

        // 山の幅は下に行くほど広がる
        let half_width = (dy as f32 * 0.6) as u32;

        for dx in 0..=half_width * 2 {
            let x = (peak_x + dx).saturating_sub(half_width);
            if x >= TILE_SIZE {
                continue;
            }

            // 左側は暗く、右側は明るく（立体感）
            let is_left = dx < half_width;
            let is_snow_zone = dy < snow_line;

            let color = if is_snow_zone {
                // 雪ゾーン
                if is_left {
                    snow_shadow
                } else if rng.gen_bool(0.3) {
                    snow_shadow
                } else {
                    snow
                }
            } else {
                // 岩ゾーン
                let r: f32 = rng.r#gen();
                if is_left {
                    // 左側（影）
                    if r < 0.6 { rock_dark } else { rock_mid }
                } else {
                    // 右側（光）
                    if r < 0.3 { rock_light } else if r < 0.6 { rock_mid } else { rock_dark }
                }
            };

            img.put_pixel(x, y, color);
        }
    }
}

fn generate_player(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);

    // 透明で初期化
    let transparent = Rgba([0, 0, 0, 0]);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, transparent);
        }
    }

    // カラーパレット（勇者風）
    let skin = Rgba([255, 210, 170, 255]);       // 肌色
    let hair = Rgba([220, 180, 80, 255]);        // 金髪
    let hair_dark = Rgba([180, 140, 60, 255]);   // 髪の影
    let eye = Rgba([40, 60, 120, 255]);          // 青い目
    let armor = Rgba([180, 180, 200, 255]);      // 鎧（シルバー）
    let armor_light = Rgba([220, 220, 240, 255]);// 鎧ハイライト
    let armor_dark = Rgba([120, 120, 140, 255]); // 鎧の影
    let cape_red = Rgba([180, 50, 50, 255]);     // 赤マント
    let cape_dark = Rgba([140, 30, 30, 255]);    // マントの影
    let gold = Rgba([255, 215, 0, 255]);         // 金の装飾
    let sword = Rgba([200, 210, 220, 255]);      // 剣（銀）
    let sword_light = Rgba([240, 245, 255, 255]);// 剣ハイライト
    let boots = Rgba([80, 60, 40, 255]);         // ブーツ

    // 剣（右手側）- x: 12-13
    img.put_pixel(13, 2, sword_light);
    img.put_pixel(13, 3, sword);
    img.put_pixel(13, 4, sword);
    img.put_pixel(13, 5, sword);
    img.put_pixel(13, 6, sword);
    img.put_pixel(13, 7, gold);  // 剣の柄
    img.put_pixel(12, 7, gold);
    img.put_pixel(14, 7, gold);
    img.put_pixel(13, 8, Rgba([100, 70, 40, 255])); // グリップ

    // 髪の毛（金髪、逆立ち気味）- y: 1-4
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

    // 顔 - y: 5-7
    for x in 5..=10 {
        img.put_pixel(x, 5, skin);
    }
    // 目のある行（キリッとした目）
    img.put_pixel(5, 6, skin);
    img.put_pixel(6, 6, eye);
    img.put_pixel(7, 6, skin);
    img.put_pixel(8, 6, skin);
    img.put_pixel(9, 6, eye);
    img.put_pixel(10, 6, skin);
    // 口元
    for x in 5..=10 {
        img.put_pixel(x, 7, skin);
    }

    // 鎧（上半身）- y: 8-11
    // 肩アーマー
    img.put_pixel(3, 8, armor_dark);
    img.put_pixel(4, 8, armor);
    img.put_pixel(11, 8, armor);
    img.put_pixel(12, 8, armor_light);

    for x in 5..=10 {
        img.put_pixel(x, 8, if x < 8 { armor_dark } else { armor });
    }
    // 胸部（金の装飾）
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

    // マント（背中から左右に）
    img.put_pixel(2, 9, cape_dark);
    img.put_pixel(2, 10, cape_red);
    img.put_pixel(2, 11, cape_red);
    img.put_pixel(3, 10, cape_dark);
    img.put_pixel(3, 11, cape_red);

    // 腕（鎧）
    img.put_pixel(3, 9, armor);
    img.put_pixel(4, 10, skin);  // 手
    img.put_pixel(12, 9, armor_light);
    img.put_pixel(11, 10, skin); // 手（剣を持つ）

    // 腰ベルト - y: 12
    for x in 5..=10 {
        img.put_pixel(x, 12, if x == 7 || x == 8 { gold } else { armor_dark });
    }

    // 脚（鎧） - y: 13
    img.put_pixel(5, 13, armor_dark);
    img.put_pixel(6, 13, armor);
    img.put_pixel(9, 13, armor);
    img.put_pixel(10, 13, armor_light);

    // マント下部
    img.put_pixel(2, 12, cape_red);
    img.put_pixel(2, 13, cape_dark);
    img.put_pixel(3, 12, cape_dark);
    img.put_pixel(3, 13, cape_red);

    // ブーツ - y: 14
    img.put_pixel(5, 14, boots);
    img.put_pixel(6, 14, boots);
    img.put_pixel(9, 14, boots);
    img.put_pixel(10, 14, boots);

    img.save(output_dir.join("player.png")).expect("Failed to save player.png");
    println!("Generated: player.png");
}
