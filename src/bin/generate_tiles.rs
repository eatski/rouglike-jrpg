use image::{ImageBuffer, Rgba, RgbaImage};
use rand::Rng;
use std::fs;
use std::path::Path;

const TILE_SIZE: u32 = 16;

fn main() {
    let output_dir = Path::new("assets/tiles");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    // 各地形タイルを生成
    generate_sea(output_dir);
    generate_plains(output_dir);
    generate_forest(output_dir);
    generate_mountain(output_dir);

    println!("Tiles generated in {}", output_dir.display());
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
