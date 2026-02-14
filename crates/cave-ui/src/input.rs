use bevy::prelude::*;

use cave::{try_cave_move, CaveMoveResult, CaveTerrain};

use app_state::AppState;
use movement_ui::{
    MovementBlockedEvent, MovementLocked, PendingMove, Player, PlayerArrivedEvent,
    PlayerMovedEvent, SmoothMove, TilePosition,
};
use shared_ui::{MovementState, TILE_SIZE};
use input_ui;

use super::scene::CaveMapResource;

const MOVE_DURATION: f32 = 0.15;

/// 洞窟内のプレイヤー移動入力を処理
#[allow(clippy::too_many_arguments)]
pub fn cave_player_movement(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    cave_map: Res<CaveMapResource>,
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

    if locked.is_some() {
        return;
    }

    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    let x_pressed = input_ui::is_x_pressed(&keyboard);
    let y_pressed = input_ui::is_y_pressed(&keyboard);
    let x_just_pressed = input_ui::is_x_just_pressed(&keyboard);
    let y_just_pressed = input_ui::is_y_just_pressed(&keyboard);

    if x_just_pressed && !y_pressed {
        move_state.first_axis = Some(true);
    } else if y_just_pressed && !x_pressed {
        move_state.first_axis = Some(false);
    } else if !x_pressed && !y_pressed {
        move_state.first_axis = None;
    }

    if input_ui::is_up_pressed(&keyboard) {
        dy = 1;
    }
    if input_ui::is_down_pressed(&keyboard) {
        dy = -1;
    }
    if input_ui::is_left_pressed(&keyboard) {
        dx = -1;
    }
    if input_ui::is_right_pressed(&keyboard) {
        dx = 1;
    }

    if dx == 0 && dy == 0 {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = (0, 0);
        return;
    }

    let (first_dx, first_dy, pending_direction) = if dx != 0 && dy != 0 {
        match move_state.first_axis {
            Some(true) => (dx, 0, Some((0, dy))),
            Some(false) => (0, dy, Some((dx, 0))),
            None => (dx, 0, Some((0, dy))),
        }
    } else {
        (dx, dy, None)
    };

    let current_direction = (first_dx, first_dy);
    let direction_changed = current_direction != move_state.last_direction;

    let should_move = if direction_changed {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = current_direction;
        true
    } else if move_state.is_repeating {
        move_state.timer.tick(time.delta());
        move_state.timer.just_finished()
    } else {
        move_state.initial_delay.tick(time.delta());
        if move_state.initial_delay.just_finished() {
            move_state.is_repeating = true;
            move_state.timer.reset();
            true
        } else {
            false
        }
    };

    if !should_move {
        return;
    }

    match try_cave_move(
        tile_pos.x,
        tile_pos.y,
        first_dx,
        first_dy,
        &cave_map.grid,
        cave_map.width,
        cave_map.height,
    ) {
        CaveMoveResult::Moved { new_x, new_y } => {
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (first_dx, first_dy),
            });
            if let Some(dir) = pending_direction {
                commands.entity(entity).insert(PendingMove { direction: dir });
            }
        }
        CaveMoveResult::Blocked => {
            blocked_events.write(MovementBlockedEvent {
                entity,
                direction: (first_dx, first_dy),
            });
        }
    }
}

/// 洞窟用スムーズ移動開始（トーラスラップなし）
pub fn start_cave_smooth_move(
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

        // 洞窟ではラップしない: final_pos == target_pos
        commands.entity(event.entity).insert((
            SmoothMove {
                from: current_pos,
                to: target_pos,
                final_pos: target_pos,
                timer: Timer::from_seconds(MOVE_DURATION, TimerMode::Once),
            },
            MovementLocked,
        ));
    }
}

/// 洞窟用スムーズ移動更新
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_cave_smooth_move(
    mut commands: Commands,
    time: Res<Time>,
    cave_map: Res<CaveMapResource>,
    mut query: Query<
        (
            Entity,
            &mut SmoothMove,
            &mut Transform,
            &mut TilePosition,
            Option<&PendingMove>,
        ),
        With<Player>,
    >,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut arrived_events: MessageWriter<PlayerArrivedEvent>,
) {
    let Ok((entity, mut smooth_move, mut transform, mut tile_pos, pending_move)) =
        query.single_mut()
    else {
        return;
    };

    smooth_move.timer.tick(time.delta());

    if smooth_move.timer.just_finished() {
        transform.translation.x = smooth_move.final_pos.x;
        transform.translation.y = smooth_move.final_pos.y;

        commands.entity(entity).remove::<SmoothMove>();

        if let Some(pending) = pending_move {
            let (dx, dy) = pending.direction;
            commands.entity(entity).remove::<PendingMove>();

            match try_cave_move(
                tile_pos.x,
                tile_pos.y,
                dx,
                dy,
                &cave_map.grid,
                cave_map.width,
                cave_map.height,
            ) {
                CaveMoveResult::Moved { new_x, new_y } => {
                    tile_pos.x = new_x;
                    tile_pos.y = new_y;
                    moved_events.write(PlayerMovedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                }
                CaveMoveResult::Blocked => {
                    blocked_events.write(MovementBlockedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                    // ワープゾーン上でPendingMoveがブロックされた場合にも
                    // 到着判定を発火させる（MovementLockedはバウンスが解除）
                    arrived_events.write(PlayerArrivedEvent { entity });
                }
            }
        } else {
            commands.entity(entity).remove::<MovementLocked>();
            arrived_events.write(PlayerArrivedEvent { entity });
        }
    } else {
        let progress = smooth_move.timer.fraction();
        let eased = 1.0 - (1.0 - progress) * (1.0 - progress);
        let current_pos = smooth_move.from.lerp(smooth_move.to, eased);
        transform.translation.x = current_pos.x;
        transform.translation.y = current_pos.y;
    }
}

/// ワープゾーンに到着したらフィールドに戻る
pub fn check_warp_zone_system(
    mut events: MessageReader<PlayerArrivedEvent>,
    player_query: Query<&TilePosition, With<Player>>,
    cave_map: Res<CaveMapResource>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _event in events.read() {
        let Ok(tile_pos) = player_query.single() else {
            continue;
        };

        if tile_pos.x < cave_map.width && tile_pos.y < cave_map.height {
            let terrain = cave_map.grid[tile_pos.y][tile_pos.x];
            if terrain == CaveTerrain::WarpZone {
                next_state.set(AppState::Exploring);
                return;
            }
        }
    }
}
