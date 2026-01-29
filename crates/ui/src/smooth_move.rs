use bevy::prelude::*;

use crate::components::{Boat, MovementLocked, OnBoat, PendingMove, Player, TilePosition};
use crate::events::{MovementBlockedEvent, PlayerMovedEvent};
use crate::movement_helpers::{execute_boat_move, execute_walk_move};
use crate::resources::MapDataResource;

use super::constants::{MAP_PIXEL_HEIGHT, MAP_PIXEL_WIDTH, TILE_SIZE};

/// 移動アニメーションの持続時間（秒）
const MOVE_DURATION: f32 = 0.15;

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

        // アニメーション用の目標座標（ラップしない、移動方向に素直に動く）
        let target_pos = current_pos + Vec2::new(dx as f32 * TILE_SIZE, dy as f32 * TILE_SIZE);

        // ラップ後の最終座標を計算
        let half_width = MAP_PIXEL_WIDTH / 2.0;
        let half_height = MAP_PIXEL_HEIGHT / 2.0;

        let mut final_pos = target_pos;
        if final_pos.x > half_width {
            final_pos.x -= MAP_PIXEL_WIDTH;
        } else if final_pos.x < -half_width {
            final_pos.x += MAP_PIXEL_WIDTH;
        }
        if final_pos.y > half_height {
            final_pos.y -= MAP_PIXEL_HEIGHT;
        } else if final_pos.y < -half_height {
            final_pos.y += MAP_PIXEL_HEIGHT;
        }

        commands.entity(event.entity).insert((
            SmoothMove {
                from: current_pos,
                to: target_pos,
                final_pos,
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
    map_data: Res<MapDataResource>,
    mut query: Query<
        (
            Entity,
            &mut SmoothMove,
            &mut Transform,
            &mut TilePosition,
            Option<&PendingMove>,
            Option<&OnBoat>,
        ),
        With<Player>,
    >,
    mut boat_query: Query<(Entity, &mut TilePosition), (With<Boat>, Without<Player>)>,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
) {
    let Ok((entity, mut smooth_move, mut transform, mut tile_pos, pending_move, on_boat)) =
        query.single_mut()
    else {
        return;
    };

    smooth_move.timer.tick(time.delta());

    if smooth_move.timer.just_finished() {
        // 移動完了、ラップ後の最終位置にセット
        transform.translation.x = smooth_move.final_pos.x;
        transform.translation.y = smooth_move.final_pos.y;

        // コンポーネント削除
        commands.entity(entity).remove::<SmoothMove>();

        // PendingMoveがあれば2回目の移動を試行
        if let Some(pending) = pending_move {
            let (dx, dy) = pending.direction;
            commands.entity(entity).remove::<PendingMove>();

            let move_success = if let Some(on_boat) = on_boat {
                // 船での2回目移動
                matches!(
                    execute_boat_move(
                        &mut commands, entity, &mut tile_pos, dx, dy,
                        &map_data.grid, on_boat, &mut boat_query,
                        &mut moved_events, &mut blocked_events,
                    ),
                    crate::movement_helpers::ExecuteMoveResult::Success
                )
            } else {
                // 徒歩での2回目移動
                matches!(
                    execute_walk_move(
                        &mut tile_pos, entity, dx, dy,
                        &map_data.grid, &mut moved_events, &mut blocked_events,
                    ),
                    crate::movement_helpers::ExecuteMoveResult::Success
                )
            };

            // 2回目移動が成功した場合、MovementLockedは維持（新しいSmoothMoveが追加される）
            // 失敗した場合、バウンスが開始されるのでMovementLockedは維持
            if !move_success {
                // バウンスが始まるのでロック維持（バウンス終了時に解除）
            }
            // 成功した場合はstart_smooth_moveで新しいSmoothMoveが追加される
        } else {
            // PendingMoveがなければロック解除
            commands.entity(entity).remove::<MovementLocked>();
        }
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
