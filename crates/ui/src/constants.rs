use game::map::{MAP_HEIGHT, MAP_WIDTH};

// === タイル・マップ表示 ===
pub const TILE_SIZE: f32 = 4.0;
pub const MAP_PIXEL_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;
pub const MAP_PIXEL_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_SIZE;
pub const PLAYER_SIZE: f32 = TILE_SIZE * 0.7;
pub const VISIBLE_CELLS: f32 = 9.0;
pub const VISIBLE_SIZE: f32 = VISIBLE_CELLS * TILE_SIZE;
pub const WINDOW_SCALE: f32 = 16.0;
pub const WINDOW_SIZE: f32 = VISIBLE_SIZE * WINDOW_SCALE;
pub const CULLING_MARGIN: f32 = TILE_SIZE; // カリング判定のマージン

// === 移動タイミング ===
/// キー押しっぱなし時のリピート間隔（秒）
/// 60ms = 約16.7歩/秒 - キビキビした移動感
pub const MOVEMENT_REPEAT_INTERVAL: f32 = 0.06;
/// 移動開始までの初回遅延（秒）
/// 150ms - 誤入力防止とレスポンスのバランス
pub const MOVEMENT_INITIAL_DELAY: f32 = 0.15;

/// タイル座標をワールド座標に変換
pub fn tile_to_world(tile_x: usize, tile_y: usize) -> (f32, f32) {
    let origin_x = -MAP_PIXEL_WIDTH / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -MAP_PIXEL_HEIGHT / 2.0 + TILE_SIZE / 2.0;
    (
        origin_x + tile_x as f32 * TILE_SIZE,
        origin_y + tile_y as f32 * TILE_SIZE,
    )
}

/// 論理座標（負の値を許容）をワールド座標に変換
///
/// タイルプールなど、マップ端を超えた論理座標を扱う場合に使用。
pub fn logical_to_world(logical_x: i32, logical_y: i32) -> (f32, f32) {
    let origin_x = -MAP_PIXEL_WIDTH / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -MAP_PIXEL_HEIGHT / 2.0 + TILE_SIZE / 2.0;
    (
        origin_x + logical_x as f32 * TILE_SIZE,
        origin_y + logical_y as f32 * TILE_SIZE,
    )
}
