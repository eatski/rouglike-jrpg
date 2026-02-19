use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_noise, save_image, TILE_SIZE};

pub fn generate_chest(output_dir: &Path) {
    let mut img = new_image();

    // --- パレット ---
    let transparent = Rgba([0, 0, 0, 0]);
    let shadow = Rgba([40, 35, 28, 255]);

    let outline = Rgba([50, 30, 10, 255]);
    let wood_dark = Rgba([100, 60, 20, 255]);
    let wood_mid = Rgba([140, 85, 35, 255]);
    let wood_light = Rgba([170, 110, 50, 255]);
    let wood_highlight = Rgba([195, 140, 70, 255]);
    let lid_top = Rgba([210, 155, 85, 255]);

    let gold_dark = Rgba([170, 130, 15, 255]);
    let gold_mid = Rgba([210, 175, 40, 255]);
    let gold_light = Rgba([245, 215, 60, 255]);
    let gold_shine = Rgba([255, 240, 140, 255]);

    let keyhole = Rgba([30, 20, 8, 255]);

    // --- 全ピクセル透明で初期化 ---
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, transparent);
        }
    }

    // 宝箱の配置: 中央に置く
    // 箱体: x=2..13 (12px幅), y=7..14 (8px高)
    // 蓋:   x=2..13 (12px幅), y=2..7  (6px高, アーチ型)

    // --- 影（箱の底面に沿って） ---
    for x in 3..14 {
        img.put_pixel(x, 15, shadow);
    }
    for x in 4..13 {
        img.put_pixel(x, 14, shadow);
    }

    // === 箱体部分 (y=8..14) ===
    for y in 8..=14 {
        for x in 2..=13 {
            let is_left = x == 2;
            let is_right = x == 13;
            let is_bottom = y == 14;

            if is_left || is_right || is_bottom {
                img.put_pixel(x, y, outline);
            } else if x == 3 {
                // 左側面の暗い部分
                img.put_pixel(x, y, wood_dark);
            } else if x == 12 {
                // 右側面のやや暗い部分
                let n = pixel_noise(x, y, 200);
                img.put_pixel(x, y, if n > 0.5 { wood_dark } else { wood_mid });
            } else {
                // 木目テクスチャ
                let n = pixel_noise(x, y, 201);
                if n > 0.8 {
                    img.put_pixel(x, y, wood_light);
                } else if n > 0.25 {
                    img.put_pixel(x, y, wood_mid);
                } else {
                    img.put_pixel(x, y, wood_dark);
                }
            }
        }
    }

    // === 蓋部分 (アーチ型) ===
    // 蓋の行ごとの横幅（アーチを表現）
    // y=2: x=5..10 (6px)  -- 頂点
    // y=3: x=4..11 (8px)
    // y=4: x=3..12 (10px)
    // y=5: x=2..13 (12px)
    // y=6: x=2..13 (12px)
    // y=7: x=2..13 (12px) -- 蓋の底辺
    let lid_rows: [(u32, u32, u32); 6] = [
        // (y, x_start, x_end_inclusive)
        (2, 5, 10),
        (3, 4, 11),
        (4, 3, 12),
        (5, 2, 13),
        (6, 2, 13),
        (7, 2, 13),
    ];

    for &(y, x_start, x_end) in &lid_rows {
        for x in x_start..=x_end {
            let is_left = x == x_start;
            let is_right = x == x_end;
            let is_top = y == 2;
            let is_bottom_lid = y == 7;

            if is_left || is_right || is_top {
                img.put_pixel(x, y, outline);
            } else if is_bottom_lid {
                // 蓋底辺（金属バンドを後で描画）
                img.put_pixel(x, y, outline);
            } else if y == 3 && (x == x_start + 1 || x == x_end - 1) {
                // アーチの内側角
                img.put_pixel(x, y, outline);
            } else {
                // 蓋の内部: 上が明るく下が暗いグラデーション
                let brightness = pixel_noise(x, y, 150);
                if y <= 3 {
                    img.put_pixel(x, y, lid_top);
                } else if y == 4 {
                    img.put_pixel(
                        x,
                        y,
                        if brightness > 0.6 {
                            lid_top
                        } else {
                            wood_highlight
                        },
                    );
                } else if y == 5 {
                    img.put_pixel(
                        x,
                        y,
                        if brightness > 0.7 {
                            wood_highlight
                        } else {
                            wood_light
                        },
                    );
                } else {
                    // y == 6
                    img.put_pixel(
                        x,
                        y,
                        if brightness > 0.5 {
                            wood_light
                        } else {
                            wood_mid
                        },
                    );
                }
            }
        }
    }

    // アーチの角をさらに丸くする
    img.put_pixel(5, 2, transparent);
    img.put_pixel(10, 2, transparent);

    // === 金属バンド（蓋と箱体の境目） ===
    for x in 3..=12 {
        img.put_pixel(x, 7, gold_dark);
    }
    // バンドのハイライト
    for x in 4..=8 {
        img.put_pixel(x, 7, gold_mid);
    }

    // === 中央バックル ===
    // バックルの金具枠: x=6..9, y=5..10
    for y in 5..=10 {
        for x in 6..=9 {
            let is_edge = x == 6 || x == 9 || y == 5 || y == 10;
            if is_edge {
                img.put_pixel(x, y, gold_dark);
            } else {
                img.put_pixel(x, y, gold_light);
            }
        }
    }
    // バックル内のハイライト
    img.put_pixel(7, 6, gold_shine);
    img.put_pixel(7, 7, gold_mid);
    img.put_pixel(8, 7, gold_mid);

    // 鍵穴
    img.put_pixel(7, 9, keyhole);
    img.put_pixel(8, 9, keyhole);
    img.put_pixel(7, 8, gold_shine);
    img.put_pixel(8, 8, gold_light);

    // === 蓋の縦バンド（左右） ===
    for &(y, x_start, x_end) in &lid_rows {
        if y >= 3 && y <= 7 {
            let bl = x_start + 2;
            let br = x_end - 2;
            if bl >= x_start + 1 && bl < 6 {
                img.put_pixel(bl, y, gold_dark);
            }
            if br <= x_end - 1 && br > 9 {
                img.put_pixel(br, y, gold_dark);
            }
        }
    }
    // 箱体の縦バンド
    for y in 8..=13 {
        img.put_pixel(4, y, gold_dark);
        img.put_pixel(11, y, gold_dark);
    }

    // === 箱体のハイライト（左上に光沢感） ===
    img.put_pixel(4, 8, gold_mid);
    img.put_pixel(11, 8, gold_mid);

    save_image(&img, output_dir, "chest.png");
}

pub fn generate_chest_open(output_dir: &Path) {
    let mut img = new_image();

    // --- パレット（閉じた宝箱と同一） ---
    let transparent = Rgba([0, 0, 0, 0]);
    let shadow = Rgba([40, 35, 28, 255]);

    let outline = Rgba([50, 30, 10, 255]);
    let wood_dark = Rgba([100, 60, 20, 255]);
    let wood_mid = Rgba([140, 85, 35, 255]);
    let wood_light = Rgba([170, 110, 50, 255]);
    let wood_highlight = Rgba([195, 140, 70, 255]);

    let gold_dark = Rgba([170, 130, 15, 255]);
    let gold_mid = Rgba([210, 175, 40, 255]);

    // 箱の内側の色
    let interior_dark = Rgba([60, 35, 12, 255]);
    let interior_mid = Rgba([80, 50, 20, 255]);
    let interior_light = Rgba([100, 65, 30, 255]);

    // --- 全ピクセル透明で初期化 ---
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            img.put_pixel(x, y, transparent);
        }
    }

    // 開いた宝箱の配置:
    // 蓋(後方に倒れた状態): x=2..13, y=1..4 (薄い帯状、奥に倒れている)
    // 箱体:                 x=2..13, y=8..14 (閉じた宝箱と同じ)
    // 箱の内側:             x=3..12, y=5..7  (開口部、内部が見える)

    // --- 影（箱の底面に沿って） ---
    for x in 3..14 {
        img.put_pixel(x, 15, shadow);
    }
    for x in 4..13 {
        img.put_pixel(x, 14, shadow);
    }

    // === 箱体部分 (y=8..14) -- 閉じた宝箱と同じ ===
    for y in 8..=14 {
        for x in 2..=13 {
            let is_left = x == 2;
            let is_right = x == 13;
            let is_bottom = y == 14;

            if is_left || is_right || is_bottom {
                img.put_pixel(x, y, outline);
            } else if x == 3 {
                img.put_pixel(x, y, wood_dark);
            } else if x == 12 {
                let n = pixel_noise(x, y, 200);
                img.put_pixel(x, y, if n > 0.5 { wood_dark } else { wood_mid });
            } else {
                let n = pixel_noise(x, y, 201);
                if n > 0.8 {
                    img.put_pixel(x, y, wood_light);
                } else if n > 0.25 {
                    img.put_pixel(x, y, wood_mid);
                } else {
                    img.put_pixel(x, y, wood_dark);
                }
            }
        }
    }

    // === 箱体の縦バンド（金属） ===
    for y in 8..=13 {
        img.put_pixel(4, y, gold_dark);
        img.put_pixel(11, y, gold_dark);
    }
    img.put_pixel(4, 8, gold_mid);
    img.put_pixel(11, 8, gold_mid);

    // === 箱の内側（開口部 y=5..7） ===
    // 上辺アウトライン (y=5)
    for x in 2..=13 {
        img.put_pixel(x, 5, outline);
    }
    // 内側 (y=6..7)
    for y in 6..=7 {
        for x in 2..=13 {
            let is_left = x == 2;
            let is_right = x == 13;

            if is_left || is_right {
                img.put_pixel(x, y, outline);
            } else {
                let n = pixel_noise(x, y, 220);
                if y == 6 {
                    // 上側: やや明るい（光が差し込む）
                    img.put_pixel(
                        x,
                        y,
                        if n > 0.6 {
                            interior_light
                        } else {
                            interior_mid
                        },
                    );
                } else {
                    // 下側: 暗め
                    img.put_pixel(
                        x,
                        y,
                        if n > 0.7 {
                            interior_mid
                        } else {
                            interior_dark
                        },
                    );
                }
            }
        }
    }
    // 内側の金属バンド位置にも暗い色
    img.put_pixel(4, 6, interior_dark);
    img.put_pixel(4, 7, interior_dark);
    img.put_pixel(11, 6, interior_dark);
    img.put_pixel(11, 7, interior_dark);

    // === 蓋と箱体の境目（金属バンド y=8 上端） ===
    for x in 3..=12 {
        img.put_pixel(x, 8, gold_dark);
    }
    for x in 4..=8 {
        img.put_pixel(x, 8, gold_mid);
    }

    // === 蓋（後方に倒れた状態 y=1..4） ===
    // 蓋は奥に倒れているため、正面から見ると薄い帯状に見える
    // y=1: x=4..11 (上辺、アーチの頂点が潰れた形)
    // y=2: x=3..12
    // y=3: x=2..13
    // y=4: x=2..13 (蓋の手前端 = ヒンジ部分)
    let lid_open_rows: [(u32, u32, u32); 4] = [
        (1, 4, 11),
        (2, 3, 12),
        (3, 2, 13),
        (4, 2, 13),
    ];

    for &(y, x_start, x_end) in &lid_open_rows {
        for x in x_start..=x_end {
            let is_left = x == x_start;
            let is_right = x == x_end;
            let is_top = y == 1;

            if is_left || is_right || is_top {
                img.put_pixel(x, y, outline);
            } else if y == 4 {
                // ヒンジ部分（金属バンド）
                img.put_pixel(x, y, gold_dark);
            } else {
                // 蓋の裏面（内側が見えている、暗めの木目）
                let n = pixel_noise(x, y, 230);
                if y == 2 {
                    img.put_pixel(
                        x,
                        y,
                        if n > 0.5 {
                            wood_highlight
                        } else {
                            wood_light
                        },
                    );
                } else {
                    // y == 3
                    img.put_pixel(
                        x,
                        y,
                        if n > 0.6 {
                            wood_light
                        } else {
                            wood_mid
                        },
                    );
                }
            }
        }
    }

    // 蓋の角を丸くする
    img.put_pixel(4, 1, transparent);
    img.put_pixel(11, 1, transparent);

    // 蓋のヒンジバンドのハイライト
    for x in 4..=8 {
        img.put_pixel(x, 4, gold_mid);
    }

    // 蓋の縦バンド（左右の金属）
    for &(y, x_start, x_end) in &lid_open_rows {
        if y >= 2 && y <= 4 {
            let bl = x_start + 2;
            let br = x_end - 2;
            if bl >= x_start + 1 && bl < 6 {
                img.put_pixel(bl, y, gold_dark);
            }
            if br <= x_end - 1 && br > 9 {
                img.put_pixel(br, y, gold_dark);
            }
        }
    }

    save_image(&img, output_dir, "chest_open.png");
}
