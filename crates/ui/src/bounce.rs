use bevy::prelude::*;

use crate::components::{MovementLocked, Player};
use crate::events::MovementBlockedEvent;

use super::constants::TILE_SIZE;

const BOUNCE_DISTANCE: f32 = TILE_SIZE * 0.3;
const BOUNCE_DURATION: f32 = 0.12;

#[derive(Component)]
pub struct Bounce {
    pub direction: Vec2,
    pub timer: Timer,
    pub offset: Vec2,
}

/// 移動ブロックイベントを受け取ってバウンスを開始
pub fn start_bounce(mut commands: Commands, mut events: MessageReader<MovementBlockedEvent>) {
    for event in events.read() {
        let dir = Vec2::new(event.direction.0 as f32, event.direction.1 as f32).normalize();
        commands.entity(event.entity).insert((
            Bounce {
                direction: dir,
                timer: Timer::from_seconds(BOUNCE_DURATION, TimerMode::Once),
                offset: Vec2::ZERO,
            },
            MovementLocked,
        ));
    }
}

/// バウンスアニメーションを更新
pub fn update_bounce(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Bounce, &mut Transform), With<Player>>,
) {
    let Ok((entity, mut bounce, mut transform)) = query.single_mut() else {
        return;
    };

    // 前回のオフセットを元に戻す
    transform.translation.x -= bounce.offset.x;
    transform.translation.y -= bounce.offset.y;

    bounce.timer.tick(time.delta());

    if bounce.timer.just_finished() {
        // バウンス終了、ロック解除
        commands.entity(entity).remove::<Bounce>();
        commands.entity(entity).remove::<MovementLocked>();
    } else {
        // バウンスアニメーション（往復）
        let progress = bounce.timer.fraction();
        // sin波で往復: 0→1→0
        let bounce_factor = (progress * std::f32::consts::PI).sin();
        let new_offset = bounce.direction * BOUNCE_DISTANCE * bounce_factor;

        transform.translation.x += new_offset.x;
        transform.translation.y += new_offset.y;
        bounce.offset = new_offset;
    }
}
