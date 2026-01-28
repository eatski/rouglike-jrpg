use bevy::prelude::*;

use crate::components::Player;
use crate::events::PlayerMovedEvent;

use super::constants::{MAP_PIXEL_HEIGHT, MAP_PIXEL_WIDTH, TILE_SIZE};

/// 移動イベントを受け取ってプレイヤーのワールド座標を更新
pub fn update_player_position(
    mut events: MessageReader<PlayerMovedEvent>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    for event in events.read() {
        let Ok(mut transform) = query.get_mut(event.entity) else {
            continue;
        };

        let (dx, dy) = event.direction;

        // ワールド座標を更新
        transform.translation.x += dx as f32 * TILE_SIZE;
        transform.translation.y += dy as f32 * TILE_SIZE;

        // 中央マップの範囲を超えたらワールド座標をラップ
        let half_width = MAP_PIXEL_WIDTH / 2.0;
        let half_height = MAP_PIXEL_HEIGHT / 2.0;

        if transform.translation.x > half_width {
            transform.translation.x -= MAP_PIXEL_WIDTH;
        } else if transform.translation.x < -half_width {
            transform.translation.x += MAP_PIXEL_WIDTH;
        }

        if transform.translation.y > half_height {
            transform.translation.y -= MAP_PIXEL_HEIGHT;
        } else if transform.translation.y < -half_height {
            transform.translation.y += MAP_PIXEL_HEIGHT;
        }
    }
}
