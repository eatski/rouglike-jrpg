use bevy::prelude::*;

use crate::components::{MovementLocked, Player};
use crate::events::PlayerMovedEvent;

use super::constants::{MAP_PIXEL_HEIGHT, MAP_PIXEL_WIDTH, TILE_SIZE};

/// 移動アニメーションの持続時間（秒）
const MOVE_DURATION: f32 = 0.15;

/// 滑らか移動コンポーネント
#[derive(Component)]
pub struct SmoothMove {
    /// 移動元の座標
    pub from: Vec2,
    /// 移動先の座標
    pub to: Vec2,
    /// アニメーションタイマー
    pub timer: Timer,
}

/// 移動イベントを受け取って滑らか移動を開始
pub fn start_smooth_move(
    mut commands: Commands,
    mut events: MessageReader<PlayerMovedEvent>,
    query: Query<&Transform, With<Player>>,
) {
    for event in events.read() {
        let Ok(transform) = query.get(event.entity) else {
            continue;
        };

        let (dx, dy) = event.direction;
        let current_pos = Vec2::new(transform.translation.x, transform.translation.y);

        // 目標座標を計算
        let mut target_pos = current_pos + Vec2::new(dx as f32 * TILE_SIZE, dy as f32 * TILE_SIZE);

        // 中央マップの範囲を超える場合のラップ処理
        let half_width = MAP_PIXEL_WIDTH / 2.0;
        let half_height = MAP_PIXEL_HEIGHT / 2.0;

        if target_pos.x > half_width {
            target_pos.x -= MAP_PIXEL_WIDTH;
        } else if target_pos.x < -half_width {
            target_pos.x += MAP_PIXEL_WIDTH;
        }

        if target_pos.y > half_height {
            target_pos.y -= MAP_PIXEL_HEIGHT;
        } else if target_pos.y < -half_height {
            target_pos.y += MAP_PIXEL_HEIGHT;
        }

        commands.entity(event.entity).insert((
            SmoothMove {
                from: current_pos,
                to: target_pos,
                timer: Timer::from_seconds(MOVE_DURATION, TimerMode::Once),
            },
            MovementLocked,
        ));
    }
}

/// 滑らか移動アニメーションを更新
pub fn update_smooth_move(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SmoothMove, &mut Transform), With<Player>>,
) {
    let Ok((entity, mut smooth_move, mut transform)) = query.single_mut() else {
        return;
    };

    smooth_move.timer.tick(time.delta());

    if smooth_move.timer.just_finished() {
        // 移動完了、最終位置にセット
        transform.translation.x = smooth_move.to.x;
        transform.translation.y = smooth_move.to.y;

        // コンポーネント削除してロック解除
        commands.entity(entity).remove::<SmoothMove>();
        commands.entity(entity).remove::<MovementLocked>();
    } else {
        // 線形補間で滑らかに移動
        let progress = smooth_move.timer.fraction();
        // イージング関数を適用（ease-out）
        let eased_progress = ease_out_quad(progress);

        let current_pos = smooth_move.from.lerp(smooth_move.to, eased_progress);
        transform.translation.x = current_pos.x;
        transform.translation.y = current_pos.y;
    }
}

/// Ease-out quadratic イージング関数
/// 移動開始時は速く、終了時はゆっくり
fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}
