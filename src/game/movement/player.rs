use bevy::prelude::*;

use crate::game::map::{MapData, Terrain, MAP_HEIGHT, MAP_WIDTH};

use super::events::{MovementBlockedEvent, PlayerMovedEvent};

#[derive(Component)]
pub struct Player;

/// 移動処理中かどうかを示すマーカーコンポーネント（UI側で設定）
#[derive(Component)]
pub struct MovementLocked;

#[derive(Component)]
pub struct TilePosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Resource)]
pub struct SpawnPosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Resource)]
pub struct MovementState {
    pub timer: Timer,
    pub initial_delay: Timer,
    pub is_repeating: bool,
    pub last_direction: (i32, i32),
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.08, TimerMode::Repeating),
            initial_delay: Timer::from_seconds(0.2, TimerMode::Once),
            is_repeating: false,
            last_direction: (0, 0),
        }
    }
}

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map_data: Res<MapData>,
    mut move_state: ResMut<MovementState>,
    mut query: Query<
        (Entity, &mut TilePosition, Option<&MovementLocked>),
        With<Player>,
    >,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
) {
    let Ok((entity, mut tile_pos, locked)) = query.single_mut() else {
        return;
    };

    // 移動ロック中は入力を無視
    if locked.is_some() {
        return;
    }

    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        dy = 1;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dy = -1;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dx = -1;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dx = 1;
    }

    let current_direction = (dx, dy);

    // 方向キーが押されていない場合はリセット
    if dx == 0 && dy == 0 {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = (0, 0);
        return;
    }

    // 方向が変わったか判定（新しい入力として扱う）
    let direction_changed = current_direction != move_state.last_direction;

    let should_move = if direction_changed {
        // 方向変更時は即座に移動、タイマーリセット
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = current_direction;
        true
    } else if move_state.is_repeating {
        // リピート中は通常のタイマーで移動
        move_state.timer.tick(time.delta());
        move_state.timer.just_finished()
    } else {
        // 初回遅延を待つ
        move_state.initial_delay.tick(time.delta());
        if move_state.initial_delay.just_finished() {
            move_state.is_repeating = true;
            move_state.timer.reset();
            true
        } else {
            false
        }
    };

    if should_move {
        // 移動先のタイル座標を計算
        let new_x = ((tile_pos.x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
        let new_y = ((tile_pos.y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;

        // 海には移動できない
        if map_data.grid[new_y][new_x] == Terrain::Sea {
            blocked_events.write(MovementBlockedEvent {
                entity,
                direction: (dx, dy),
            });
            return;
        }

        // タイル位置を更新
        tile_pos.x = new_x;
        tile_pos.y = new_y;

        // 移動イベントを発行（UI側で座標更新を処理）
        moved_events.write(PlayerMovedEvent {
            entity,
            direction: (dx, dy),
        });
    }
}
