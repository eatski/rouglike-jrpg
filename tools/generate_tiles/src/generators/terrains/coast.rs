use image::Rgba;
use std::path::Path;

use crate::generators::common::{new_image, pixel_noise, save_image};

// --- 海岸ルックアップ（ビットマスク・象限・正規化） ---

const N: u8 = 1;
const NE: u8 = 2;
const E: u8 = 4;
const SE: u8 = 8;
const S: u8 = 16;
const SW: u8 = 32;
const W: u8 = 64;
const NW: u8 = 128;

#[derive(Clone, Copy, PartialEq)]
enum QuadrantState {
    Sea,
    EdgeNS,
    EdgeWE,
    InnerCorner,
    Inner,
}

fn normalize_mask(mask: u8) -> u8 {
    let mut result = mask & (N | E | S | W);
    if mask & NE != 0 && mask & N != 0 && mask & E != 0 { result |= NE; }
    if mask & SE != 0 && mask & S != 0 && mask & E != 0 { result |= SE; }
    if mask & SW != 0 && mask & S != 0 && mask & W != 0 { result |= SW; }
    if mask & NW != 0 && mask & N != 0 && mask & W != 0 { result |= NW; }
    result
}

fn quadrant_state(cardinal_ns: bool, cardinal_we: bool, diagonal: bool) -> QuadrantState {
    match (cardinal_ns, cardinal_we, diagonal) {
        (false, false, _) => QuadrantState::Sea,
        (true, false, _) => QuadrantState::EdgeNS,
        (false, true, _) => QuadrantState::EdgeWE,
        (true, true, false) => QuadrantState::InnerCorner,
        (true, true, true) => QuadrantState::Inner,
    }
}

fn mask_to_quadrants(mask: u8) -> [QuadrantState; 4] {
    let (n, e, s, w) = (mask & N != 0, mask & E != 0, mask & S != 0, mask & W != 0);
    let (ne, se, sw, nw) = (mask & NE != 0, mask & SE != 0, mask & SW != 0, mask & NW != 0);
    [
        quadrant_state(n, w, nw),
        quadrant_state(n, e, ne),
        quadrant_state(s, w, sw),
        quadrant_state(s, e, se),
    ]
}

/// ルックアップテーブル構築（ユニークマスク一覧付き）
fn build_lookup_table() -> (Vec<u8>, usize) {
    let mut unique_masks: Vec<u8> = Vec::new();
    for raw in 0..=255u8 {
        let normalized = normalize_mask(raw);
        if !unique_masks.contains(&normalized) {
            unique_masks.push(normalized);
        }
    }
    let count = unique_masks.len();
    (unique_masks, count)
}

// --- 描画ロジック ---

/// 深海パレット（sea.rsと同一）
const SEA_BASE: Rgba<u8> = Rgba([40, 80, 120, 255]);
const SEA_LIGHT: Rgba<u8> = Rgba([60, 120, 180, 255]);
const SEA_HIGHLIGHT: Rgba<u8> = Rgba([100, 160, 220, 255]);

/// 浅瀬パレット（深海より明るめ、同じ波パターンを使用）
const SHALLOW_BASE: Rgba<u8> = Rgba([60, 115, 155, 255]);
const SHALLOW_LIGHT: Rgba<u8> = Rgba([80, 145, 195, 255]);
const SHALLOW_HIGHLIGHT: Rgba<u8> = Rgba([115, 175, 225, 255]);

fn sea_pixel(x: u32, y: u32) -> Rgba<u8> {
    let wave = ((y as f32 / 4.0).sin() * 2.0 + (x as f32 / 3.0).cos()) as i32;
    if (y as i32 + wave) % 6 < 2 {
        SEA_LIGHT
    } else if pixel_noise(x, y, 1) < 0.05 {
        SEA_HIGHLIGHT
    } else {
        SEA_BASE
    }
}

fn shallow_pixel(x: u32, y: u32) -> Rgba<u8> {
    let wave = ((y as f32 / 4.0).sin() * 2.0 + (x as f32 / 3.0).cos()) as i32;
    if (y as i32 + wave) % 6 < 2 {
        SHALLOW_LIGHT
    } else if pixel_noise(x, y, 1) < 0.05 {
        SHALLOW_HIGHLIGHT
    } else {
        SHALLOW_BASE
    }
}

fn quadrant_distance(state: QuadrantState, lx: f32, ly: f32, qi: usize) -> f32 {
    match state {
        QuadrantState::Sea => -100.0,
        QuadrantState::Inner => 100.0,
        QuadrantState::EdgeNS => match qi {
            0 | 1 => 4.5 - ly,
            _ => ly - 3.5,
        },
        QuadrantState::EdgeWE => match qi {
            0 | 2 => 4.5 - lx,
            _ => lx - 3.5,
        },
        QuadrantState::InnerCorner => {
            let (cx, cy) = match qi {
                0 => (-0.5_f32, -0.5_f32),
                1 => (8.5, -0.5),
                2 => (-0.5, 8.5),
                _ => (8.5, 8.5),
            };
            ((lx - cx).powi(2) + (ly - cy).powi(2)).sqrt() - 3.0
        }
    }
}

/// 47枚の海岸タイルを生成する
pub fn generate_coast_tiles(output_dir: &Path) {
    let (unique_masks, count) = build_lookup_table();

    for tile_idx in 0..count {
        let quadrants = mask_to_quadrants(unique_masks[tile_idx]);
        let mut img = new_image();

        for gy in 0..16u32 {
            for gx in 0..16u32 {
                let qi = match (gx >= 8, gy >= 8) {
                    (false, false) => 0,
                    (true, false) => 1,
                    (false, true) => 2,
                    (true, true) => 3,
                };
                let lx = (gx % 8) as f32;
                let ly = (gy % 8) as f32;
                let dist = quadrant_distance(quadrants[qi], lx, ly, qi);
                let adjusted = dist + (pixel_noise(gx, gy, 70) - 0.5) * 0.8;

                let color = if adjusted > 0.0 {
                    shallow_pixel(gx, gy)
                } else {
                    sea_pixel(gx, gy)
                };
                img.put_pixel(gx, gy, color);
            }
        }

        save_image(&img, output_dir, &format!("coast_{:03}.png", tile_idx));
    }

    println!("Generated {} coast tiles", count);
}
