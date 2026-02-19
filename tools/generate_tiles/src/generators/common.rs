use image::{ImageBuffer, RgbaImage};
use std::path::Path;

pub const TILE_SIZE: u32 = 16;

/// TILE_SIZE x TILE_SIZE の RGBA 画像を作成（全ピクセル透明で初期化）
pub fn new_image() -> RgbaImage {
    ImageBuffer::new(TILE_SIZE, TILE_SIZE)
}

/// 画像を保存して生成メッセージを表示
pub fn save_image(img: &RgbaImage, output_dir: &Path, filename: &str) {
    img.save(output_dir.join(filename))
        .unwrap_or_else(|_| panic!("Failed to save {filename}"));
    println!("Generated: {filename}");
}

/// 座標から決定的にハッシュ値を返す (0..256)
pub fn pixel_hash(x: u32, y: u32, salt: u32) -> u32 {
    let mut h = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))
        .wrapping_add(salt.wrapping_mul(2147483647));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    h & 0xFF
}

/// pixel_hash を 0.0..1.0 の f32 に変換
pub fn pixel_noise(x: u32, y: u32, salt: u32) -> f32 {
    pixel_hash(x, y, salt) as f32 / 255.0
}
