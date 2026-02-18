use image::{ImageBuffer, Rgba, RgbaImage};
use rand::Rng;
use std::fs;
use std::path::Path;

const TILE_SIZE: u32 = 16;

fn main() {
    let tiles_dir = Path::new("assets/tiles");
    let chars_dir = Path::new("assets/characters");
    let enemies_dir = Path::new("assets/enemies");
    fs::create_dir_all(tiles_dir).expect("Failed to create tiles directory");
    fs::create_dir_all(chars_dir).expect("Failed to create characters directory");
    fs::create_dir_all(enemies_dir).expect("Failed to create enemies directory");

    // 各地形タイルを生成
    generate_sea(tiles_dir);
    generate_plains(tiles_dir);
    generate_forest(tiles_dir);
    generate_mountain(tiles_dir);
    generate_boat(tiles_dir);
    generate_town(tiles_dir);
    generate_cave(tiles_dir);

    // 洞窟内部タイルを生成
    generate_cave_wall(tiles_dir);
    generate_cave_floor(tiles_dir);
    generate_warp_zone(tiles_dir);
    generate_ladder(tiles_dir);

    // キャラクターを生成
    generate_player(chars_dir);

    // 敵キャラクターを生成
    generate_slime(enemies_dir);
    generate_bat(enemies_dir);
    generate_goblin(enemies_dir);
    generate_wolf(enemies_dir);
    generate_ghost(enemies_dir);

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

    // 岩場カラーパレット（茶色・グレー系の岩）
    let bg = Rgba([80, 100, 70, 255]);           // 背景（草っぽい地面）
    let rock_dark = Rgba([80, 70, 60, 255]);     // 暗い岩（茶色がかったグレー）
    let rock_mid = Rgba([120, 110, 100, 255]);   // 中間の岩
    let rock_light = Rgba([160, 150, 135, 255]); // 明るい岩（ハイライト）
    let rock_edge = Rgba([60, 55, 50, 255]);     // 岩の輪郭・影
    let moss = Rgba([70, 90, 60, 255]);          // 苔（登れそうな感じを演出）

    // 背景で埋める
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, bg);
        }
    }

    // ゴツゴツした岩の塊を複数配置（登れそうな岩場）
    let rock_colors = RockColors { dark: rock_dark, mid: rock_mid, light: rock_light, edge: rock_edge, moss };

    // メインの大きな岩（中央下寄り）
    draw_rocky_boulder(&mut img, &mut rng, 6, 5, 8, 10, &rock_colors);

    // 小さな岩（左上）
    draw_rocky_boulder(&mut img, &mut rng, 1, 2, 5, 6, &rock_colors);

    // 小さな岩（右上）
    draw_rocky_boulder(&mut img, &mut rng, 11, 3, 4, 5, &rock_colors);

    // 足場になる小石を散らす
    for _ in 0..8 {
        let x = rng.gen_range(0..TILE_SIZE);
        let y = rng.gen_range(10..TILE_SIZE);
        let color = if rng.gen_bool(0.5) { rock_mid } else { rock_dark };
        if img.get_pixel(x, y) == &bg {
            img.put_pixel(x, y, color);
        }
    }

    img.save(output_dir.join("mountain.png")).expect("Failed to save mountain.png");
    println!("Generated: mountain.png");
}

struct RockColors {
    dark: Rgba<u8>,
    mid: Rgba<u8>,
    light: Rgba<u8>,
    edge: Rgba<u8>,
    moss: Rgba<u8>,
}

/// ゴツゴツした岩の塊を描画
fn draw_rocky_boulder(
    img: &mut RgbaImage,
    rng: &mut impl rand::Rng,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    colors: &RockColors,
) {
    let RockColors { dark: rock_dark, mid: rock_mid, light: rock_light, edge: rock_edge, moss } = *colors;
    // 岩のベース形状を描画
    for dy in 0..height {
        let y = start_y + dy;
        if y >= TILE_SIZE {
            break;
        }

        // 岩の形状：中央が膨らんで端が狭まる不規則な形
        let progress = dy as f32 / height as f32;
        let bulge = if progress < 0.3 {
            // 上部：やや狭い
            (progress / 0.3 * 0.8 + 0.2) * width as f32
        } else if progress < 0.7 {
            // 中央部：最も広い
            width as f32
        } else {
            // 下部：やや狭まる
            ((1.0 - progress) / 0.3 * 0.6 + 0.4) * width as f32
        };

        let current_width = bulge as u32;
        let offset = (width - current_width) / 2;

        for dx in 0..current_width {
            let x = start_x + offset + dx;
            if x >= TILE_SIZE {
                continue;
            }

            // 不規則な凹凸を加える
            let noise = rng.gen_range(0..3);
            if noise == 0 && dx == 0 {
                continue; // 左端を欠けさせる
            }
            if noise == 1 && dx == current_width - 1 {
                continue; // 右端を欠けさせる
            }

            // 岩の色を決定（立体感+ランダム性）
            let is_left_edge = dx == 0;
            let is_right_edge = dx == current_width - 1;
            let is_top = dy < 2;
            let is_bottom = dy >= height - 2;

            let color = if is_left_edge || is_bottom {
                // 左端・下端は輪郭（影）
                rock_edge
            } else if is_right_edge || is_top {
                // 右端・上端はハイライト
                if rng.gen_bool(0.6) { rock_light } else { rock_mid }
            } else {
                // 内側：ランダムに色を変えてゴツゴツ感を出す
                let r: f32 = rng.r#gen();
                if r < 0.1 {
                    moss // 苔（登れそうな雰囲気）
                } else if r < 0.3 {
                    rock_light
                } else if r < 0.6 {
                    rock_mid
                } else {
                    rock_dark
                }
            };

            img.put_pixel(x, y, color);
        }
    }

    // 岩の表面にひび割れ・凹凸を追加
    for _ in 0..3 {
        let cx = start_x + rng.gen_range(1..width.saturating_sub(1));
        let cy = start_y + rng.gen_range(1..height.saturating_sub(1));
        if cx < TILE_SIZE && cy < TILE_SIZE {
            img.put_pixel(cx, cy, rock_edge);
        }
    }
}

fn generate_boat(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);

    // 透明で初期化
    let transparent = Rgba([0, 0, 0, 0]);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, transparent);
        }
    }

    // 船のカラーパレット
    let wood_dark = Rgba([100, 60, 30, 255]);    // 暗い木
    let wood_mid = Rgba([140, 90, 50, 255]);     // 中間の木
    let wood_light = Rgba([180, 130, 80, 255]);  // 明るい木
    let sail_white = Rgba([240, 240, 230, 255]); // 帆（白）
    let sail_shadow = Rgba([200, 200, 190, 255]); // 帆の影
    let mast = Rgba([120, 80, 40, 255]);         // マスト

    // 船体（下部、楕円形）- y: 10-15
    // 船首（左）から船尾（右）
    for y in 10..=14 {
        let row_width = match y {
            10 => (4, 12),  // 上部：狭い
            11 => (3, 13),
            12 => (2, 14),
            13 => (2, 14),
            14 => (3, 13),  // 底部：やや狭い
            _ => (4, 12),
        };
        for x in row_width.0..=row_width.1 {
            let color = if x <= 4 || y == 10 {
                wood_light  // 船首・上部はハイライト
            } else if x >= 12 || y >= 13 {
                wood_dark   // 船尾・底部は影
            } else {
                wood_mid
            };
            img.put_pixel(x, y, color);
        }
    }

    // 船の縁（上部の輪郭）
    for x in 4..=12 {
        img.put_pixel(x, 9, wood_dark);
    }

    // マスト（中央） - x: 8
    for y in 3..=9 {
        img.put_pixel(8, y, mast);
    }

    // 帆（三角形、左向き） - y: 3-8
    for y in 3..=8 {
        let sail_width = (y - 2).min(5);
        for dx in 1..=sail_width {
            let x = 8 - dx;
            if x >= 3 {
                let color = if dx == 1 { sail_shadow } else { sail_white };
                img.put_pixel(x, y, color);
            }
        }
    }

    // 帆の右側（小さい）
    for y in 4..=7 {
        let sail_width = ((y - 3) as i32).min(3);
        for dx in 1..=sail_width {
            let x = 8 + dx as u32;
            if x <= 11 {
                let color = if dx == 1 { sail_shadow } else { sail_white };
                img.put_pixel(x, y, color);
            }
        }
    }

    // 旗（マストの上）
    img.put_pixel(8, 2, Rgba([200, 50, 50, 255]));  // 赤い旗
    img.put_pixel(9, 2, Rgba([200, 50, 50, 255]));
    img.put_pixel(9, 1, Rgba([180, 40, 40, 255]));

    img.save(output_dir.join("boat.png")).expect("Failed to save boat.png");
    println!("Generated: boat.png");
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

fn generate_slime(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);

    // 透明で初期化
    let transparent = Rgba([0, 0, 0, 0]);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, transparent);
        }
    }

    // スライムのカラーパレット（青/水色系、半透明感）
    let slime_dark = Rgba([40, 100, 180, 255]);     // 濃い青（影・輪郭）
    let slime_mid = Rgba([60, 140, 220, 255]);      // 中間の青（本体）
    let slime_light = Rgba([100, 180, 240, 255]);   // 明るい青（ハイライト）
    let slime_bright = Rgba([150, 220, 255, 255]);  // 最も明るい（光沢）
    let eye_white = Rgba([255, 255, 255, 255]);     // 目の白
    let eye_black = Rgba([0, 0, 0, 255]);           // 目の黒（瞳）

    // スライムの本体（ぷるぷる感を出す楕円形）
    // 下部を広く、上部をやや狭く（お餅っぽい形）

    // 最上部（y: 4-5）
    for x in 6..=9 {
        img.put_pixel(x, 4, slime_dark);  // 輪郭
    }
    for x in 7..=8 {
        img.put_pixel(x, 5, slime_mid);
    }

    // 上部（y: 6-7、目のある部分）
    for x in 5..=10 {
        let color = if x == 5 || x == 10 {
            slime_dark  // 輪郭
        } else if x == 6 || x == 9 {
            slime_mid   // 本体
        } else {
            slime_light // 内側
        };
        img.put_pixel(x, 6, color);
    }

    // 目（y: 7）
    img.put_pixel(5, 7, slime_dark);   // 左輪郭
    img.put_pixel(6, 7, eye_white);    // 左目白
    img.put_pixel(7, 7, slime_light);  // 中央ハイライト
    img.put_pixel(8, 7, slime_light);
    img.put_pixel(9, 7, eye_white);    // 右目白
    img.put_pixel(10, 7, slime_dark);  // 右輪郭

    // 瞳（y: 8）
    img.put_pixel(4, 8, slime_dark);   // 左輪郭
    img.put_pixel(5, 8, slime_mid);
    img.put_pixel(6, 8, eye_black);    // 左瞳
    img.put_pixel(7, 8, slime_light);
    img.put_pixel(8, 8, slime_light);
    img.put_pixel(9, 8, eye_black);    // 右瞳
    img.put_pixel(10, 8, slime_mid);
    img.put_pixel(11, 8, slime_dark);  // 右輪郭

    // 中央部（y: 9-10、最も膨らんでいる）
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            slime_dark  // 輪郭
        } else if x == 5 || x == 10 {
            slime_mid
        } else if x == 7 || x == 8 {
            slime_bright  // 中央ハイライト（ぷるぷる感）
        } else {
            slime_light
        };
        img.put_pixel(x, 9, color);
    }
    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            slime_dark  // 輪郭
        } else if x == 4 || x == 11 {
            slime_mid
        } else if x == 7 || x == 8 {
            slime_bright  // 光沢
        } else {
            slime_light
        };
        img.put_pixel(x, 10, color);
    }

    // 下部（y: 11-12、やや狭まる）
    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            slime_dark
        } else if x == 4 || x == 11 {
            slime_mid
        } else {
            slime_light
        };
        img.put_pixel(x, 11, color);
    }
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            slime_dark
        } else if x == 5 || x == 10 {
            slime_mid
        } else {
            slime_light
        };
        img.put_pixel(x, 12, color);
    }

    // 底部（y: 13-14、地面と接する）
    for x in 5..=10 {
        let color = if x == 5 || x == 10 {
            slime_dark
        } else {
            slime_mid
        };
        img.put_pixel(x, 13, color);
    }
    // 最下部の影
    for x in 6..=9 {
        img.put_pixel(x, 14, slime_dark);
    }

    // 追加のハイライト（光沢感を強調）
    img.put_pixel(6, 6, slime_bright);  // 頭部の光沢
    img.put_pixel(7, 6, slime_bright);
    img.put_pixel(6, 9, slime_bright);  // 本体の光沢
    img.put_pixel(9, 10, slime_bright); // 本体右側の光沢

    img.save(output_dir.join("slime.png")).expect("Failed to save slime.png");
    println!("Generated: slime.png");
}

fn generate_cave(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);
    let mut rng = rand::thread_rng();

    // 洞窟タイルのカラーパレット
    let rock_bg = Rgba([100, 95, 85, 255]);         // 背景（岩場）
    let rock_dark = Rgba([70, 65, 55, 255]);        // 暗い岩（影）
    let rock_mid = Rgba([120, 110, 95, 255]);       // 中間の岩
    let rock_light = Rgba([150, 140, 120, 255]);    // 明るい岩（ハイライト）
    let cave_black = Rgba([20, 20, 25, 255]);       // 洞窟入口（真っ暗）
    let cave_dark = Rgba([40, 40, 45, 255]);        // 洞窟入口の周辺（暗い）
    let moss = Rgba([60, 80, 50, 255]);             // 苔（洞窟周辺の湿気）

    // 背景（岩場）で埋める
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r: f32 = rng.r#gen();
            let color = if r < 0.2 {
                rock_dark
            } else if r < 0.3 {
                rock_light
            } else {
                rock_bg
            };
            img.put_pixel(x, y, color);
        }
    }

    // 洞窟の入口（アーチ形、中央下寄り）
    // 入口の形状を描画（上部が丸いアーチ）
    let cave_center_x = 8;

    // アーチの形状を定義（y座標ごとの横幅）
    let arch_shape = [
        (4, 3),   // y=4: 幅3（上部）
        (5, 5),   // y=5: 幅5
        (6, 7),   // y=6: 幅7
        (7, 8),   // y=7: 幅8
        (8, 9),   // y=8: 幅9
        (9, 9),   // y=9: 幅9
        (10, 9),  // y=10: 幅9
        (11, 9),  // y=11: 幅9
        (12, 9),  // y=12: 幅9
        (13, 9),  // y=13: 幅9
        (14, 9),  // y=14: 幅9
        (15, 9),  // y=15: 幅9
    ];

    // 洞窟入口を描画（内側は暗い）
    for (y, width) in arch_shape.iter() {
        if *y >= TILE_SIZE {
            continue;
        }
        let half_width = *width / 2;
        for dx in 0..=*width {
            let x = cave_center_x - half_width + dx;
            if x >= TILE_SIZE {
                continue;
            }

            // 入口の内側（暗い）
            let is_edge = dx == 0 || dx == *width;

            let color = if is_edge {
                rock_dark  // 入口の縁は暗い岩
            } else if *y <= 5 {
                cave_black  // 上部（奥）は真っ暗
            } else if *y <= 8 {
                cave_dark   // 中間は少し明るい
            } else {
                cave_black  // 下部も真っ暗
            };

            img.put_pixel(x, *y, color);
        }
    }

    // 洞窟の入口周辺に岩の枠を描画（より立体的に）
    // 上部のアーチ形の岩
    for y in 3..=7 {
        let arch_width = match y {
            3 => 5,
            4 => 7,
            5 => 9,
            6 => 10,
            7 => 11,
            _ => 11,
        };
        let half_width = arch_width / 2;

        for dx in [0, arch_width] {
            let x = cave_center_x - half_width + dx;
            if x < TILE_SIZE && y < TILE_SIZE {
                // 岩の縁にハイライトと影を追加
                let color = if dx == 0 {
                    rock_dark  // 左側は影
                } else {
                    rock_light  // 右側はハイライト
                };
                img.put_pixel(x, y, color);

                // 岩の厚みを追加（外側にもう1列）
                if dx == 0 && x > 0 {
                    img.put_pixel(x - 1, y, rock_mid);
                } else if dx == arch_width && x + 1 < TILE_SIZE {
                    img.put_pixel(x + 1, y, rock_light);
                }
            }
        }
    }

    // 洞窟の左右に縦の岩柱を追加
    for y in 8..TILE_SIZE {
        // 左側の岩柱
        for x in 2..=3 {
            if x < TILE_SIZE && y < TILE_SIZE {
                let color = if x == 2 { rock_dark } else { rock_mid };
                img.put_pixel(x, y, color);
            }
        }
        // 右側の岩柱
        for x in 12..=13 {
            if x < TILE_SIZE && y < TILE_SIZE {
                let color = if x == 13 { rock_light } else { rock_mid };
                img.put_pixel(x, y, color);
            }
        }
    }

    // 洞窟上部に岩の突起を追加
    // 左上の突起
    img.put_pixel(5, 2, rock_mid);
    img.put_pixel(6, 2, rock_light);
    img.put_pixel(5, 3, rock_dark);

    // 右上の突起
    img.put_pixel(9, 2, rock_light);
    img.put_pixel(10, 2, rock_mid);
    img.put_pixel(10, 3, rock_light);

    // 中央上部の突起
    img.put_pixel(7, 1, rock_dark);
    img.put_pixel(8, 1, rock_mid);
    img.put_pixel(9, 1, rock_mid);
    img.put_pixel(7, 2, rock_mid);
    img.put_pixel(8, 2, rock_dark);

    // 洞窟入口周辺に苔を追加（湿気を表現）
    let moss_positions = [
        (4, 8), (4, 9), (4, 10),
        (12, 9), (12, 10), (12, 11),
        (6, 13), (7, 14), (9, 14),
    ];

    for (mx, my) in moss_positions {
        if mx < TILE_SIZE && my < TILE_SIZE
            && rng.gen_bool(0.7)
        {
            img.put_pixel(mx, my, moss);
        }
    }

    // 入口の縁にランダムな岩の凹凸を追加
    for y in 8..13 {
        for dx in [3, 4, 11, 12] {
            if dx < TILE_SIZE && y < TILE_SIZE
                && rng.gen_bool(0.3)
            {
                let color = if dx < 8 { rock_dark } else { rock_light };
                img.put_pixel(dx, y, color);
            }
        }
    }

    img.save(output_dir.join("cave.png")).expect("Failed to save cave.png");
    println!("Generated: cave.png");
}

fn generate_town(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);
    let mut rng = rand::thread_rng();

    // 町タイルのカラーパレット（暖色系、レンガや石造りの建物）
    let grass_base = Rgba([90, 140, 80, 255]);        // 地面（草地）
    let grass_dark = Rgba([70, 110, 60, 255]);        // 草の影
    let brick = Rgba([160, 90, 70, 255]);             // レンガ（赤茶色）
    let brick_dark = Rgba([120, 70, 50, 255]);        // レンガの影
    let brick_light = Rgba([190, 110, 85, 255]);      // レンガのハイライト
    let roof_brown = Rgba([100, 60, 40, 255]);        // 屋根（茶色）
    let roof_highlight = Rgba([140, 90, 60, 255]);    // 屋根のハイライト
    let wall_beige = Rgba([220, 200, 170, 255]);      // 壁（ベージュ）
    let wall_light = Rgba([240, 230, 210, 255]);      // 壁のハイライト
    let wall_dark = Rgba([180, 160, 140, 255]);       // 壁の影
    let window = Rgba([100, 120, 140, 255]);          // 窓（青灰色）
    let door = Rgba([80, 50, 30, 255]);               // ドア（濃い茶色）

    // 背景（草地）で埋める
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let color = if rng.gen_bool(0.2) { grass_dark } else { grass_base };
            img.put_pixel(x, y, color);
        }
    }

    // メインの建物（中央の家/城）を描画
    // 建物は左側と右側に2つ配置して、町の雰囲気を出す

    // 左側の建物（小さい家） - x: 1-6
    draw_house(&mut img, &mut rng, 1, 3, 5, 10, brick, brick_dark, brick_light, roof_brown, roof_highlight, wall_beige, wall_light, wall_dark, window, door);

    // 右側の建物（城風） - x: 10-14
    draw_castle(&mut img, &mut rng, 10, 2, 5, 11, brick, brick_dark, brick_light, roof_brown, roof_highlight, wall_beige, wall_light, wall_dark, window);

    // 地面に道や装飾を追加
    // 中央に石畳の道
    let stone_light = Rgba([170, 170, 160, 255]);
    let stone_dark = Rgba([130, 130, 120, 255]);
    for y in 12..TILE_SIZE {
        for x in 6..=9 {
            let color = if rng.gen_bool(0.4) { stone_dark } else { stone_light };
            img.put_pixel(x, y, color);
        }
    }

    img.save(output_dir.join("town.png")).expect("Failed to save town.png");
    println!("Generated: town.png");
}

/// 家を描画（左側の小さい建物）
#[allow(clippy::too_many_arguments)]
fn draw_house(
    img: &mut RgbaImage,
    _rng: &mut impl rand::Rng,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    _brick: Rgba<u8>,
    _brick_dark: Rgba<u8>,
    _brick_light: Rgba<u8>,
    roof_brown: Rgba<u8>,
    roof_highlight: Rgba<u8>,
    wall_beige: Rgba<u8>,
    wall_light: Rgba<u8>,
    wall_dark: Rgba<u8>,
    window: Rgba<u8>,
    door: Rgba<u8>,
) {
    // 屋根（三角形） - 上部2-3行
    let roof_height = 2;
    for dy in 0..roof_height {
        let y = start_y + dy;
        if y >= TILE_SIZE {
            break;
        }
        let roof_width = (roof_height - dy) * 2 + 1;
        let offset = (width - roof_width) / 2;
        for dx in 0..roof_width {
            let x = start_x + offset + dx;
            if x >= TILE_SIZE {
                continue;
            }
            let color = if dy == 0 { roof_highlight } else { roof_brown };
            img.put_pixel(x, y, color);
        }
    }

    // 壁（本体）
    let wall_start = start_y + roof_height;
    for dy in 0..(height - roof_height) {
        let y = wall_start + dy;
        if y >= TILE_SIZE {
            break;
        }
        for dx in 0..width {
            let x = start_x + dx;
            if x >= TILE_SIZE {
                continue;
            }
            // 左端は影、右端はハイライト
            let color = if dx == 0 {
                wall_dark
            } else if dx == width - 1 {
                wall_light
            } else {
                wall_beige
            };
            img.put_pixel(x, y, color);
        }
    }

    // 窓（上部）
    let window_y = wall_start + 1;
    if window_y < TILE_SIZE && start_x + 2 < TILE_SIZE {
        img.put_pixel(start_x + 2, window_y, window);
        img.put_pixel(start_x + 3, window_y, window);
    }

    // ドア（下部）
    let door_y_start = wall_start + (height - roof_height) - 3;
    for dy in 0..3 {
        let y = door_y_start + dy;
        if y >= TILE_SIZE {
            break;
        }
        if start_x + 2 < TILE_SIZE {
            img.put_pixel(start_x + 2, y, door);
        }
        if start_x + 3 < TILE_SIZE && dy < 2 {
            img.put_pixel(start_x + 3, y, door);
        }
    }
}

fn generate_cave_wall(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);
    let mut rng = rand::thread_rng();

    let dark = Rgba([30, 28, 35, 255]);
    let mid = Rgba([50, 45, 55, 255]);
    let light = Rgba([70, 65, 75, 255]);
    let highlight = Rgba([85, 80, 90, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r: f32 = rng.r#gen();
            let color = if r < 0.3 {
                dark
            } else if r < 0.6 {
                mid
            } else if r < 0.9 {
                light
            } else {
                highlight
            };
            img.put_pixel(x, y, color);
        }
    }

    // ひび割れ
    for _ in 0..4 {
        let cx = rng.gen_range(2..TILE_SIZE - 2);
        let cy = rng.gen_range(2..TILE_SIZE - 2);
        img.put_pixel(cx, cy, Rgba([20, 18, 25, 255]));
        if cx + 1 < TILE_SIZE {
            img.put_pixel(cx + 1, cy, Rgba([25, 22, 30, 255]));
        }
    }

    img.save(output_dir.join("cave_wall.png"))
        .expect("Failed to save cave_wall.png");
    println!("Generated: cave_wall.png");
}

fn generate_cave_floor(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);
    let mut rng = rand::thread_rng();

    let base = Rgba([80, 75, 65, 255]);
    let dark = Rgba([60, 55, 48, 255]);
    let light = Rgba([100, 95, 82, 255]);
    let pebble = Rgba([110, 105, 90, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r: f32 = rng.r#gen();
            let color = if r < 0.15 {
                dark
            } else if r < 0.25 {
                light
            } else if r < 0.30 {
                pebble
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    img.save(output_dir.join("cave_floor.png"))
        .expect("Failed to save cave_floor.png");
    println!("Generated: cave_floor.png");
}

fn generate_warp_zone(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);

    let base = Rgba([60, 55, 48, 255]);
    let glow_outer = Rgba([80, 60, 160, 255]);
    let glow_mid = Rgba([120, 90, 220, 255]);
    let glow_inner = Rgba([180, 150, 255, 255]);
    let glow_center = Rgba([220, 210, 255, 255]);

    let cx = TILE_SIZE as f32 / 2.0;
    let cy = TILE_SIZE as f32 / 2.0;
    let max_radius = 7.0f32;

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let dx = x as f32 - cx + 0.5;
            let dy = y as f32 - cy + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();

            let color = if dist < 2.0 {
                glow_center
            } else if dist < 3.5 {
                glow_inner
            } else if dist < 5.0 {
                glow_mid
            } else if dist < max_radius {
                glow_outer
            } else {
                base
            };
            img.put_pixel(x, y, color);
        }
    }

    img.save(output_dir.join("warp_zone.png"))
        .expect("Failed to save warp_zone.png");
    println!("Generated: warp_zone.png");
}

/// 城風の建物を描画（右側のタワー風）
#[allow(clippy::too_many_arguments)]
fn draw_castle(
    img: &mut RgbaImage,
    _rng: &mut impl rand::Rng,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    brick: Rgba<u8>,
    brick_dark: Rgba<u8>,
    brick_light: Rgba<u8>,
    _roof_brown: Rgba<u8>,
    _roof_highlight: Rgba<u8>,
    _wall_beige: Rgba<u8>,
    _wall_light: Rgba<u8>,
    _wall_dark: Rgba<u8>,
    window: Rgba<u8>,
) {
    // 城壁の上部（城壁の凹凸パターン）
    for dx in 0..width {
        let x = start_x + dx;
        if x >= TILE_SIZE {
            continue;
        }
        let y = start_y;
        if y < TILE_SIZE {
            // 凹凸パターン（ギザギザ）
            let color = if dx % 2 == 0 { brick_light } else { brick };
            img.put_pixel(x, y, color);
        }
    }

    // 本体（レンガ造りの壁）
    for dy in 1..height {
        let y = start_y + dy;
        if y >= TILE_SIZE {
            break;
        }
        for dx in 0..width {
            let x = start_x + dx;
            if x >= TILE_SIZE {
                continue;
            }
            // レンガ模様を作る
            let brick_pattern = (dy + dx) % 3;
            let color = if dx == 0 || dy == height - 1 {
                brick_dark  // 左端と底部は影
            } else if dx == width - 1 {
                brick_light // 右端はハイライト
            } else if brick_pattern == 0 {
                brick_dark  // レンガの継ぎ目
            } else {
                brick
            };
            img.put_pixel(x, y, color);
        }
    }

    // 窓（複数配置）
    let window_positions = [(start_x + 1, start_y + 3), (start_x + 3, start_y + 3), (start_x + 2, start_y + 6)];
    for (wx, wy) in window_positions {
        if wx < TILE_SIZE && wy < TILE_SIZE {
            img.put_pixel(wx, wy, window);
        }
    }
}

fn generate_bat(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);

    // 透明で初期化
    let transparent = Rgba([0, 0, 0, 0]);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, transparent);
        }
    }

    // コウモリのカラーパレット（紫/暗い色）
    let body_dark = Rgba([40, 20, 60, 255]);       // 体の暗い部分
    let body_mid = Rgba([60, 30, 90, 255]);        // 体の中間
    let wing_dark = Rgba([50, 30, 70, 255]);       // 翼の暗い部分
    let wing_mid = Rgba([70, 40, 100, 255]);       // 翼の中間
    let wing_light = Rgba([90, 50, 120, 255]);     // 翼のハイライト
    let eye_red = Rgba([200, 50, 50, 255]);        // 赤い目

    // コウモリの体（中央、小さい楕円形）- y: 7-10, x: 7-8
    for y in 7..=10 {
        for x in 7..=8 {
            img.put_pixel(x, y, body_mid);
        }
    }
    // 体の影（下部）
    img.put_pixel(7, 10, body_dark);
    img.put_pixel(8, 10, body_dark);

    // 頭部（上部） - y: 5-6
    img.put_pixel(7, 6, body_mid);
    img.put_pixel(8, 6, body_mid);
    img.put_pixel(7, 5, body_dark);
    img.put_pixel(8, 5, body_dark);

    // 目（赤く光る） - y: 6
    img.put_pixel(6, 6, eye_red);
    img.put_pixel(9, 6, eye_red);

    // 耳（尖った） - y: 4
    img.put_pixel(6, 4, body_dark);
    img.put_pixel(9, 4, body_dark);
    img.put_pixel(7, 5, body_mid);
    img.put_pixel(8, 5, body_mid);

    // 左翼（広げた状態） - x: 1-6
    // 左翼の骨組み（上部）
    for x in 2..=5 {
        img.put_pixel(x, 8, wing_dark);
    }
    // 左翼の膜（三角形状）
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

    // 左翼の上部のギザギザ
    img.put_pixel(2, 7, wing_dark);
    img.put_pixel(3, 7, wing_mid);
    img.put_pixel(4, 7, wing_light);
    img.put_pixel(5, 7, wing_mid);

    // 右翼（広げた状態） - x: 9-14
    // 右翼の骨組み（上部）
    for x in 10..=13 {
        img.put_pixel(x, 8, wing_dark);
    }
    // 右翼の膜（三角形状）
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

    // 右翼の上部のギザギザ
    img.put_pixel(10, 7, wing_mid);
    img.put_pixel(11, 7, wing_light);
    img.put_pixel(12, 7, wing_mid);
    img.put_pixel(13, 7, wing_dark);

    // 足（小さく） - y: 11
    img.put_pixel(7, 11, body_dark);
    img.put_pixel(8, 11, body_dark);

    img.save(output_dir.join("bat.png")).expect("Failed to save bat.png");
    println!("Generated: bat.png");
}

fn generate_goblin(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);

    // 透明で初期化
    let transparent = Rgba([0, 0, 0, 0]);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, transparent);
        }
    }

    // ゴブリンのカラーパレット（緑色の肌、粗末な服）
    let skin_green = Rgba([80, 120, 60, 255]);      // 緑色の肌
    let skin_dark = Rgba([60, 90, 45, 255]);        // 肌の影
    let skin_light = Rgba([100, 140, 80, 255]);     // 肌のハイライト
    let eye_red = Rgba([200, 50, 50, 255]);         // 赤い目
    let eye_yellow = Rgba([220, 200, 80, 255]);     // 黄色い目白
    let cloth_brown = Rgba([100, 70, 40, 255]);     // 粗末な服（茶色）
    let cloth_dark = Rgba([70, 50, 30, 255]);       // 服の影
    let teeth = Rgba([230, 230, 220, 255]);         // 牙（白）
    let hair_dark = Rgba([40, 35, 30, 255]);        // 黒い髪

    // 頭部（大きめ） - y: 2-7
    // てっぺん（髪）
    for x in 6..=9 {
        img.put_pixel(x, 2, hair_dark);
    }
    for x in 5..=10 {
        img.put_pixel(x, 3, if x == 5 || x == 10 { hair_dark } else { skin_green });
    }

    // 顔の上部
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            skin_dark
        } else {
            skin_green
        };
        img.put_pixel(x, 4, color);
    }

    // 目のある行 - y: 5
    img.put_pixel(4, 5, skin_dark);
    img.put_pixel(5, 5, skin_green);
    img.put_pixel(6, 5, eye_yellow);
    img.put_pixel(7, 5, eye_red);  // 左目
    img.put_pixel(8, 5, eye_red);  // 右目
    img.put_pixel(9, 5, eye_yellow);
    img.put_pixel(10, 5, skin_green);
    img.put_pixel(11, 5, skin_dark);

    // 鼻・頬 - y: 6
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            skin_dark
        } else if x == 7 || x == 8 {
            skin_light  // 鼻
        } else {
            skin_green
        };
        img.put_pixel(x, 6, color);
    }

    // 口（にやけた表情、大きい） - y: 7
    img.put_pixel(4, 7, skin_dark);
    img.put_pixel(5, 7, skin_dark);
    img.put_pixel(6, 7, teeth);     // 左の牙
    img.put_pixel(7, 7, skin_dark);
    img.put_pixel(8, 7, skin_dark);
    img.put_pixel(9, 7, teeth);     // 右の牙
    img.put_pixel(10, 7, skin_dark);
    img.put_pixel(11, 7, skin_dark);

    // 大きな耳（横に出っ張る）
    // 左耳
    img.put_pixel(3, 4, skin_light);
    img.put_pixel(2, 5, skin_green);
    img.put_pixel(3, 5, skin_light);
    img.put_pixel(3, 6, skin_dark);

    // 右耳
    img.put_pixel(12, 4, skin_light);
    img.put_pixel(12, 5, skin_light);
    img.put_pixel(13, 5, skin_green);
    img.put_pixel(12, 6, skin_dark);

    // 首 - y: 8
    for x in 6..=9 {
        img.put_pixel(x, 8, skin_green);
    }

    // 体（粗末な服） - y: 9-12
    // 肩・上半身
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            cloth_dark
        } else {
            cloth_brown
        };
        img.put_pixel(x, 9, color);
    }

    for x in 5..=10 {
        img.put_pixel(x, 10, cloth_brown);
    }
    img.put_pixel(4, 10, cloth_dark);
    img.put_pixel(11, 10, cloth_dark);

    // 腕（緑の肌）
    img.put_pixel(3, 10, skin_green);
    img.put_pixel(4, 11, skin_green);
    img.put_pixel(12, 10, skin_green);
    img.put_pixel(11, 11, skin_green);

    // 手（握りこぶし）
    img.put_pixel(3, 11, skin_dark);
    img.put_pixel(12, 11, skin_dark);

    // 下半身
    for x in 5..=10 {
        img.put_pixel(x, 11, cloth_dark);
    }

    // 脚（短め） - y: 12-13
    img.put_pixel(5, 12, cloth_dark);
    img.put_pixel(6, 12, skin_green);
    img.put_pixel(9, 12, skin_green);
    img.put_pixel(10, 12, cloth_dark);

    // 足
    img.put_pixel(5, 13, skin_dark);
    img.put_pixel(6, 13, skin_dark);
    img.put_pixel(9, 13, skin_dark);
    img.put_pixel(10, 13, skin_dark);

    img.save(output_dir.join("goblin.png")).expect("Failed to save goblin.png");
    println!("Generated: goblin.png");
}

fn generate_wolf(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);

    // 透明で初期化
    let transparent = Rgba([0, 0, 0, 0]);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, transparent);
        }
    }

    // 狼のカラーパレット（灰色/銀色）
    let fur_dark = Rgba([60, 60, 70, 255]);        // 濃い灰色（影）
    let fur_mid = Rgba([100, 100, 110, 255]);      // 中間の灰色
    let fur_light = Rgba([140, 140, 150, 255]);    // 明るい灰色（ハイライト）
    let fur_white = Rgba([180, 180, 190, 255]);    // 銀色（最も明るい）
    let nose_black = Rgba([30, 30, 35, 255]);      // 鼻（黒）
    let eye_yellow = Rgba([220, 200, 80, 255]);    // 黄色い目
    let eye_black = Rgba([0, 0, 0, 255]);          // 瞳（黒）
    let teeth = Rgba([230, 230, 220, 255]);        // 牙（白）

    // 横向きの狼（左向き）
    // 頭部 - y: 4-8, x: 2-7

    // 耳（尖った三角形） - y: 2-4
    // 左耳（後ろ側）
    img.put_pixel(4, 2, fur_dark);
    img.put_pixel(4, 3, fur_mid);
    img.put_pixel(5, 3, fur_dark);

    // 右耳（前側）
    img.put_pixel(6, 2, fur_light);
    img.put_pixel(6, 3, fur_mid);
    img.put_pixel(7, 3, fur_light);

    // 頭の上部 - y: 4
    for x in 4..=7 {
        let color = if x == 4 {
            fur_dark
        } else if x == 7 {
            fur_light
        } else {
            fur_mid
        };
        img.put_pixel(x, 4, color);
    }

    // 目のある行 - y: 5
    img.put_pixel(3, 5, fur_mid);
    img.put_pixel(4, 5, fur_dark);
    img.put_pixel(5, 5, eye_yellow);
    img.put_pixel(6, 5, fur_light);
    img.put_pixel(7, 5, fur_light);

    // 瞳 - y: 6
    img.put_pixel(3, 6, fur_dark);
    img.put_pixel(4, 6, fur_mid);
    img.put_pixel(5, 6, eye_black);
    img.put_pixel(6, 6, fur_mid);
    img.put_pixel(7, 6, fur_white);

    // 鼻・口（前に突き出た形） - y: 7
    img.put_pixel(2, 7, fur_light);
    img.put_pixel(3, 7, fur_mid);
    img.put_pixel(4, 7, fur_mid);
    img.put_pixel(5, 7, fur_dark);
    img.put_pixel(6, 7, fur_white);
    img.put_pixel(7, 7, nose_black);  // 鼻先

    // 口・牙 - y: 8
    img.put_pixel(2, 8, fur_dark);
    img.put_pixel(3, 8, teeth);       // 牙
    img.put_pixel(4, 8, fur_dark);
    img.put_pixel(5, 8, fur_mid);
    img.put_pixel(6, 8, fur_light);

    // 首 - y: 9
    for x in 3..=8 {
        let color = if x == 3 || x == 4 {
            fur_dark
        } else if x == 7 || x == 8 {
            fur_light
        } else {
            fur_mid
        };
        img.put_pixel(x, 9, color);
    }

    // 体（背中から腹まで） - y: 10-11
    for x in 4..=12 {
        let color = if x <= 5 {
            fur_dark
        } else if x >= 11 {
            fur_light
        } else {
            fur_mid
        };
        img.put_pixel(x, 10, color);
    }

    for x in 5..=13 {
        let color = if x <= 6 {
            fur_dark
        } else if x >= 12 {
            fur_light
        } else {
            fur_mid
        };
        img.put_pixel(x, 11, color);
    }

    // 前脚（左向きなので右側に見える） - y: 12-14
    img.put_pixel(10, 12, fur_mid);
    img.put_pixel(11, 12, fur_light);
    img.put_pixel(12, 12, fur_mid);

    img.put_pixel(10, 13, fur_dark);
    img.put_pixel(11, 13, fur_mid);
    img.put_pixel(12, 13, fur_dark);

    img.put_pixel(10, 14, fur_dark);
    img.put_pixel(12, 14, fur_dark);

    // 後脚（左向きなので左側に見える） - y: 12-14
    img.put_pixel(5, 12, fur_dark);
    img.put_pixel(6, 12, fur_mid);
    img.put_pixel(7, 12, fur_dark);

    img.put_pixel(5, 13, fur_mid);
    img.put_pixel(6, 13, fur_dark);
    img.put_pixel(7, 13, fur_mid);

    img.put_pixel(5, 14, fur_dark);
    img.put_pixel(6, 14, fur_dark);

    // 尻尾（後ろに伸びる） - y: 10-12
    img.put_pixel(13, 10, fur_mid);
    img.put_pixel(14, 10, fur_light);

    img.put_pixel(13, 11, fur_dark);
    img.put_pixel(14, 11, fur_mid);

    img.put_pixel(13, 12, fur_dark);

    img.save(output_dir.join("wolf.png")).expect("Failed to save wolf.png");
    println!("Generated: wolf.png");
}

fn generate_ghost(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);

    // 透明で初期化
    let transparent = Rgba([0, 0, 0, 0]);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, transparent);
        }
    }

    // ゴーストのカラーパレット（白/半透明感）
    let ghost_white = Rgba([240, 240, 250, 255]);   // 白（本体）
    let ghost_light = Rgba([220, 220, 240, 255]);   // 明るい白
    let ghost_mid = Rgba([200, 200, 230, 255]);     // 中間の白
    let ghost_dark = Rgba([180, 180, 220, 255]);    // 影（暗い白）
    let ghost_outline = Rgba([160, 160, 210, 255]); // 輪郭（より暗い）
    let eye_red = Rgba([200, 50, 50, 255]);         // 赤く光る目
    let eye_glow = Rgba([255, 100, 100, 255]);      // 目の光

    // ゴーストの頭部（上部、丸い） - y: 2-6
    // 最上部
    for x in 7..=8 {
        img.put_pixel(x, 2, ghost_outline);
    }

    // 頭部の膨らみ
    for x in 6..=9 {
        let color = if x == 6 || x == 9 {
            ghost_outline
        } else {
            ghost_white
        };
        img.put_pixel(x, 3, color);
    }

    for x in 5..=10 {
        let color = if x == 5 || x == 10 {
            ghost_outline
        } else if x == 6 || x == 9 {
            ghost_mid
        } else {
            ghost_white
        };
        img.put_pixel(x, 4, color);
    }

    // 目のある行 - y: 5
    img.put_pixel(4, 5, ghost_outline);
    img.put_pixel(5, 5, ghost_dark);
    img.put_pixel(6, 5, eye_glow);    // 左目の光
    img.put_pixel(7, 5, ghost_light);
    img.put_pixel(8, 5, ghost_light);
    img.put_pixel(9, 5, eye_glow);    // 右目の光
    img.put_pixel(10, 5, ghost_dark);
    img.put_pixel(11, 5, ghost_outline);

    // 瞳（赤く光る） - y: 6
    img.put_pixel(4, 6, ghost_dark);
    img.put_pixel(5, 6, ghost_mid);
    img.put_pixel(6, 6, eye_red);     // 左目
    img.put_pixel(7, 6, ghost_white);
    img.put_pixel(8, 6, ghost_white);
    img.put_pixel(9, 6, eye_red);     // 右目
    img.put_pixel(10, 6, ghost_mid);
    img.put_pixel(11, 6, ghost_dark);

    // 口のある行（にやけた感じ） - y: 7
    for x in 4..=11 {
        let color = if x == 4 || x == 11 {
            ghost_dark
        } else if (6..=9).contains(&x) {
            ghost_outline  // 口（暗い）
        } else {
            ghost_mid
        };
        img.put_pixel(x, 7, color);
    }

    // 体（上半身、やや広がる） - y: 8-10
    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            ghost_outline
        } else if x == 4 || x == 11 {
            ghost_dark
        } else if x == 5 || x == 10 {
            ghost_mid
        } else {
            ghost_white
        };
        img.put_pixel(x, 8, color);
    }

    for x in 3..=12 {
        let color = if x == 3 || x == 12 {
            ghost_dark
        } else if x == 4 || x == 11 {
            ghost_mid
        } else if x == 7 || x == 8 {
            ghost_light  // 中央のハイライト（浮遊感）
        } else {
            ghost_white
        };
        img.put_pixel(x, 9, color);
    }

    for x in 2..=13 {
        let color = if x == 2 || x == 13 {
            ghost_outline
        } else if x == 3 || x == 12 {
            ghost_dark
        } else if x == 7 || x == 8 {
            ghost_light
        } else {
            ghost_mid
        };
        img.put_pixel(x, 10, color);
    }

    // 下半身（波打つ裾、幽霊らしく） - y: 11-14
    // 裾の波模様（ギザギザ）
    for x in 2..=13 {
        let color = if x == 2 || x == 13 {
            ghost_dark
        } else {
            ghost_mid
        };
        img.put_pixel(x, 11, color);
    }

    // 波の山 - y: 12
    for x in [3, 4, 7, 8, 11, 12] {
        img.put_pixel(x, 12, ghost_dark);
    }

    // 波の谷 - y: 13
    for x in [4, 8, 12] {
        img.put_pixel(x, 13, ghost_outline);
    }

    // 最下部の影（ほとんど消えかけ） - y: 14
    for x in [5, 9] {
        img.put_pixel(x, 14, ghost_outline);
    }

    // 浮遊感を出すために体の側面にハイライト
    img.put_pixel(4, 8, ghost_light);
    img.put_pixel(11, 8, ghost_light);
    img.put_pixel(5, 9, ghost_white);
    img.put_pixel(10, 9, ghost_white);

    img.save(output_dir.join("ghost.png")).expect("Failed to save ghost.png");
    println!("Generated: ghost.png");
}

fn generate_ladder(output_dir: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(TILE_SIZE, TILE_SIZE);

    // 洞窟の床をベースにした梯子タイル
    let floor_base = Rgba([80, 75, 65, 255]);
    let wood_light = Rgba([180, 140, 80, 255]);
    let wood_mid = Rgba([140, 100, 55, 255]);
    let wood_dark = Rgba([100, 70, 35, 255]);
    let hole_dark = Rgba([30, 25, 20, 255]);
    let hole_mid = Rgba([50, 42, 35, 255]);

    // 背景を洞窟の床色で埋める
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, floor_base);
        }
    }

    // 穴（梯子が降りている先）を描画
    for y in 2..14 {
        for x in 4..12 {
            img.put_pixel(x, y, if x == 4 || x == 11 || y == 2 || y == 13 { hole_mid } else { hole_dark });
        }
    }

    // 梯子の縦棒（左右2本）
    let rail_left: u32 = 5;
    let rail_right: u32 = 10;
    for y in 1..15 {
        img.put_pixel(rail_left, y, wood_mid);
        img.put_pixel(rail_right, y, wood_mid);
    }

    // 梯子の横棒（踏み板）
    let rungs = [3, 6, 9, 12];
    for &ry in &rungs {
        for x in (rail_left + 1)..rail_right {
            img.put_pixel(x, ry, wood_light);
        }
        // 横棒の影
        if ry + 1 < TILE_SIZE {
            for x in (rail_left + 1)..rail_right {
                img.put_pixel(x, ry + 1, wood_dark);
            }
        }
    }

    img.save(output_dir.join("ladder.png"))
        .expect("Failed to save ladder.png");
    println!("Generated: ladder.png");
}
