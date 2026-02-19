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
