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
    generate_boat(tiles_dir);

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
    // メインの大きな岩（中央下寄り）
    draw_rocky_boulder(&mut img, &mut rng, 6, 5, 8, 10, rock_dark, rock_mid, rock_light, rock_edge, moss);

    // 小さな岩（左上）
    draw_rocky_boulder(&mut img, &mut rng, 1, 2, 5, 6, rock_dark, rock_mid, rock_light, rock_edge, moss);

    // 小さな岩（右上）
    draw_rocky_boulder(&mut img, &mut rng, 11, 3, 4, 5, rock_dark, rock_mid, rock_light, rock_edge, moss);

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

/// ゴツゴツした岩の塊を描画
fn draw_rocky_boulder(
    img: &mut RgbaImage,
    rng: &mut impl rand::Rng,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    rock_dark: Rgba<u8>,
    rock_mid: Rgba<u8>,
    rock_light: Rgba<u8>,
    rock_edge: Rgba<u8>,
    moss: Rgba<u8>,
) {
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
