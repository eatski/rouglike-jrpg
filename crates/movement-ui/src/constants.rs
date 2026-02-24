// === 移動タイミング ===
/// キー押しっぱなし時のリピート間隔（秒）
/// 60ms = 約16.7歩/秒 - キビキビした移動感
pub const MOVEMENT_REPEAT_INTERVAL: f32 = 0.06;
/// 移動開始までの初回遅延（秒）
/// 150ms - 誤入力防止とレスポンスのバランス
pub const MOVEMENT_INITIAL_DELAY: f32 = 0.15;
