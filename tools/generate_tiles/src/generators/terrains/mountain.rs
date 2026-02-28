use image::{Rgba, RgbaImage};
use std::path::Path;

use crate::generators::common::{new_image, pixel_hash, pixel_noise, save_image, TILE_SIZE};

// タイル上に三角形の山ピークを描画する
// peak_x: ピーク頂点のX座標, peak_y: ピーク頂点のY座標
// base_y: 山の底辺のY座標, half_base: 底辺の半幅
#[allow(clippy::too_many_arguments)]
fn draw_mountain_peak(
    img: &mut RgbaImage,
    peak_x: u32,
    peak_y: u32,
    base_y: u32,
    half_base: u32,
    salt: u32,
    rock_dark: Rgba<u8>,
    rock_mid: Rgba<u8>,
    rock_light: Rgba<u8>,
    rock_shadow: Rgba<u8>,
    snow: Rgba<u8>,
    snow_shadow: Rgba<u8>,
) {
    let height = base_y.saturating_sub(peak_y);
    if height == 0 {
        return;
    }

    for dy in 0..=height {
        let y = peak_y + dy;
        if y >= TILE_SIZE {
            break;
        }

        // 高さ比率（0.0=頂上, 1.0=麓）
        let progress = dy as f32 / height as f32;

        // 三角形の左右境界（ピークから広がる）
        let half_w = (progress * half_base as f32) as u32;
        let left = peak_x.saturating_sub(half_w);
        let right = (peak_x + half_w).min(TILE_SIZE - 1);

        // 雪線（上部30%は雪）
        let is_snow = progress < 0.30;

        for x in left..=right {
            if x >= TILE_SIZE {
                continue;
            }

            let is_left_edge = x == left;
            let is_right_edge = x == right;

            let color = if is_snow {
                // 雪エリア：左辺（影）と右辺（光）で差をつける
                if is_left_edge {
                    snow_shadow
                } else if is_right_edge {
                    snow
                } else {
                    // 雪のテクスチャ
                    let n = pixel_noise(x, y, salt + 10);
                    if n < 0.2 { snow_shadow } else { snow }
                }
            } else if is_left_edge {
                // 左辺は影（険しい崖面）
                rock_shadow
            } else if is_right_edge {
                // 右辺は少し明るい
                if pixel_noise(x, y, salt + 1) < 0.5 { rock_mid } else { rock_light }
            } else {
                // 岩の内部：ノイズでテクスチャ
                let n = pixel_noise(x, y, salt + 2);
                if n < 0.15 {
                    rock_shadow
                } else if n < 0.40 {
                    rock_dark
                } else if n < 0.70 {
                    rock_mid
                } else {
                    rock_light
                }
            };

            img.put_pixel(x, y, color);
        }
    }

    // 亀裂（クラック）を縦に2本描画して険しさを強調
    for crack_idx in 0..2u32 {
        let cx_base = pixel_hash(crack_idx, 0, salt + 20 + crack_idx) % (half_base.max(2) - 1) + 1;
        let crack_start = peak_y + height / 4;
        let crack_len = height / 3;
        for i in 0..crack_len {
            let cy = crack_start + i;
            if cy >= TILE_SIZE {
                break;
            }
            let progress_c = i as f32 / height as f32;
            let half_w_c = (progress_c * half_base as f32) as u32;
            let left_c = peak_x.saturating_sub(half_w_c);
            let cx = (left_c + cx_base).min(TILE_SIZE - 1);
            if cx < TILE_SIZE {
                img.put_pixel(cx, cy, rock_shadow);
            }
        }
    }
}

pub fn generate_mountain(output_dir: &Path) {
    let mut img = new_image();

    // --- グレー系パレット ---
    let bg         = Rgba([85, 82, 78, 255]);   // 岩盤グレー（背景）
    let rock_shadow = Rgba([50, 48, 45, 255]);  // 深い影・亀裂
    let rock_dark  = Rgba([80, 76, 72, 255]);   // 暗い岩
    let rock_mid   = Rgba([115, 110, 105, 255]);// 中間グレー
    let rock_light = Rgba([150, 145, 138, 255]);// 明るいグレー（光面）
    let snow       = Rgba([220, 222, 225, 255]);// 雪
    let snow_shadow = Rgba([170, 172, 178, 255]);// 雪の影

    // 背景を岩盤グレーで塗りつぶす
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            // 背景にも少しノイズを入れて岩の質感
            let n = pixel_noise(x, y, 1);
            let base_color = if n < 0.2 {
                rock_dark
            } else if n < 0.6 {
                bg
            } else {
                rock_mid
            };
            img.put_pixel(x, y, base_color);
        }
    }

    // --- 山ピークを3つ描画（険しい峰の連なり）---
    // 中央の主峰（一番大きく目立つ）
    draw_mountain_peak(
        &mut img,
        8,    // peak_x (中央)
        1,    // peak_y (上端近く)
        15,   // base_y (タイル底辺)
        8,    // half_base
        100,
        rock_dark, rock_mid, rock_light, rock_shadow, snow, snow_shadow,
    );

    // 左の副峰（少し小さい）
    draw_mountain_peak(
        &mut img,
        3,    // peak_x
        4,    // peak_y
        14,   // base_y
        5,    // half_base
        200,
        rock_dark, rock_mid, rock_light, rock_shadow, snow, snow_shadow,
    );

    // 右の副峰
    draw_mountain_peak(
        &mut img,
        13,   // peak_x
        5,    // peak_y
        14,   // base_y
        4,    // half_base
        300,
        rock_dark, rock_mid, rock_light, rock_shadow, snow, snow_shadow,
    );

    // --- 岩屑・破片を麓に散らす ---
    let debris: [(u32, u32); 10] = [
        (1, 13), (3, 15), (5, 14), (7, 15), (9, 13),
        (11, 14), (13, 15), (14, 13), (2, 14), (15, 14),
    ];
    for (x, y) in debris {
        let x = x.min(TILE_SIZE - 1);
        let y = y.min(TILE_SIZE - 1);
        let n = pixel_noise(x, y, 400);
        let color = if n < 0.4 { rock_shadow } else if n < 0.7 { rock_dark } else { rock_mid };
        img.put_pixel(x, y, color);
    }

    save_image(&img, output_dir, "mountain.png");
}
