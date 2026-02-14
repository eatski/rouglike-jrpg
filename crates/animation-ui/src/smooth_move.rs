use bevy::prelude::*;

/// 滑らか移動コンポーネント
#[derive(Component)]
pub struct SmoothMove {
    /// 移動元の座標
    pub from: Vec2,
    /// 移動先の座標（アニメーション用、範囲外の場合あり）
    pub to: Vec2,
    /// ラップ後の最終座標
    pub final_pos: Vec2,
    /// アニメーションタイマー
    pub timer: Timer,
}

/// Ease-out quadratic イージング関数
/// 移動開始時は速く、終了時はゆっくり
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}
