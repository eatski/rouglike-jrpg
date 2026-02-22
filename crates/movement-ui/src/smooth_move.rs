use bevy::prelude::*;

use crate::{ActiveMap, MovementLocked, Player, PlayerMovedEvent, TilePosition, TILE_SIZE};

/// 移動アニメーションの持続時間（秒）
pub const MOVE_DURATION: f32 = 0.15;

/// 滑らか移動コンポーネント
#[derive(Component)]
pub struct SmoothMove {
    /// 移動元の座標
    pub from: Vec2,
    /// 移動先の座標（アニメーション用）
    pub to: Vec2,
    /// アニメーションタイマー
    pub timer: Timer,
}

/// SmoothMoveアニメーション完了時に発行されるメッセージ
#[derive(Message)]
pub struct SmoothMoveFinishedEvent {
    pub entity: Entity,
}

/// Ease-out quadratic イージング関数
/// 移動開始時は速く、終了時はゆっくり
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// 移動イベントを受け取って滑らか移動を開始（フィールド・洞窟共通）
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
        let target_pos = current_pos + Vec2::new(dx as f32 * TILE_SIZE, dy as f32 * TILE_SIZE);

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

/// 滑らか移動アニメーションを更新（フィールド・洞窟共通）
///
/// 完了時にTilePosition + ActiveMap.to_world()でTransformを設定し、
/// float精度誤差の蓄積を防ぐ。
pub fn update_smooth_move(
    mut commands: Commands,
    time: Res<Time>,
    active_map: Res<ActiveMap>,
    mut query: Query<
        (Entity, &mut SmoothMove, &mut Transform, &TilePosition),
        With<Player>,
    >,
    mut finished_events: MessageWriter<SmoothMoveFinishedEvent>,
) {
    let Ok((entity, mut smooth_move, mut transform, tile_pos)) = query.single_mut() else {
        return;
    };

    smooth_move.timer.tick(time.delta());

    if smooth_move.timer.just_finished() {
        // TilePosition + ActiveMap.to_world() で正確な座標を設定（精度誤差を防ぐ）
        let (world_x, world_y) = active_map.to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;

        commands.entity(entity).remove::<SmoothMove>();
        finished_events.write(SmoothMoveFinishedEvent { entity });
    } else {
        let progress = smooth_move.timer.fraction();
        let eased_progress = ease_out_quad(progress);
        let current_pos = smooth_move.from.lerp(smooth_move.to, eased_progress);
        transform.translation.x = current_pos.x;
        transform.translation.y = current_pos.y;
    }
}
