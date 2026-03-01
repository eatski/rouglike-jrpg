use bevy::prelude::*;

use field_core::{ActiveMap, Player, TilePosition, TILE_SIZE};

use crate::{MovementLocked, MovementState, PlayerMovedEvent};

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

/// SmoothMoveアニメーション中かどうかを判定する。
/// 完了フレームはfalseを返す（更新を許可するため）。
pub fn is_smooth_moving(query: &Query<&SmoothMove, With<Player>>) -> bool {
    query.iter().any(|sm| !sm.timer.just_finished())
}

/// Ease-out quadratic イージング関数
/// 移動開始時は速く、終了時はゆっくり
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// 移動イベントを受け取って滑らか移動を開始（フィールド・洞窟共通）
///
/// 対角線移動サポート: 同フレームの2イベントを1つの斜め移動に統合する。
pub fn start_smooth_move(
    mut commands: Commands,
    mut events: MessageReader<PlayerMovedEvent>,
    query: Query<&Transform, With<Player>>,
) {
    let mut total_dx: i32 = 0;
    let mut total_dy: i32 = 0;
    let mut target_entity = None;

    for event in events.read() {
        total_dx += event.direction.0;
        total_dy += event.direction.1;
        target_entity = Some(event.entity);
    }

    let Some(entity) = target_entity else {
        return;
    };
    let Ok(transform) = query.get(entity) else {
        return;
    };

    let current_pos = Vec2::new(transform.translation.x, transform.translation.y);
    let target_pos =
        current_pos + Vec2::new(total_dx as f32 * TILE_SIZE, total_dy as f32 * TILE_SIZE);

    commands.entity(entity).insert((
        SmoothMove {
            from: current_pos,
            to: target_pos,
            timer: Timer::from_seconds(MOVE_DURATION, TimerMode::Once),
        },
        MovementLocked,
    ));
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
    mut move_state: ResMut<MovementState>,
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
        move_state.move_just_completed = Some(entity);
    } else {
        let progress = smooth_move.timer.fraction();
        let eased_progress = ease_out_quad(progress);
        let current_pos = smooth_move.from.lerp(smooth_move.to, eased_progress);
        transform.translation.x = current_pos.x;
        transform.translation.y = current_pos.y;
    }
}
