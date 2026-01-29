use bevy::prelude::*;

use game::map::{Terrain, MAP_HEIGHT, MAP_WIDTH};
use game::movement::{try_move, try_move_on_boat, MoveResult};

use crate::components::{Boat, MovementLocked, OnBoat, Player, TilePosition};
use crate::events::{MovementBlockedEvent, PlayerMovedEvent};
use crate::map_mode::MapModeState;
use crate::resources::{MapDataResource, MovementState};

/// プレイヤーの移動入力を処理するシステム
pub fn player_movement(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map_data: Res<MapDataResource>,
    map_mode_state: Res<MapModeState>,
    mut move_state: ResMut<MovementState>,
    mut query: Query<
        (
            Entity,
            &mut TilePosition,
            Option<&MovementLocked>,
            Option<&OnBoat>,
        ),
        With<Player>,
    >,
    mut boat_query: Query<(Entity, &mut TilePosition), (With<Boat>, Without<Player>)>,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
) {
    let Ok((entity, mut tile_pos, locked, on_boat)) = query.single_mut() else {
        return;
    };

    // 移動ロック中は入力を無視
    if locked.is_some() {
        return;
    }

    // マップモード中は移動を無効化
    if map_mode_state.enabled {
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

    if !should_move {
        return;
    }

    if let Some(on_boat) = on_boat {
        // === 船での移動 ===
        match try_move_on_boat(tile_pos.x, tile_pos.y, dx, dy, &map_data.grid) {
            MoveResult::Moved { new_x, new_y } => {
                // 船で海を移動
                if let Ok((_, mut boat_tile_pos)) = boat_query.get_mut(on_boat.boat_entity) {
                    boat_tile_pos.x = new_x;
                    boat_tile_pos.y = new_y;
                }
                tile_pos.x = new_x;
                tile_pos.y = new_y;
                moved_events.write(PlayerMovedEvent {
                    entity,
                    direction: (dx, dy),
                });
            }
            MoveResult::Blocked => {
                // 陸地にブロックされた→下船して陸地に移動
                let new_x = ((tile_pos.x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
                let new_y = ((tile_pos.y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;

                if map_data.grid[new_y][new_x] != Terrain::Sea {
                    // 下船
                    commands.entity(entity).remove::<OnBoat>();
                    tile_pos.x = new_x;
                    tile_pos.y = new_y;
                    moved_events.write(PlayerMovedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                } else {
                    blocked_events.write(MovementBlockedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                }
            }
        }
    } else {
        // === 徒歩での移動 ===
        // 移動先座標を計算
        let new_x = ((tile_pos.x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
        let new_y = ((tile_pos.y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;

        // 移動先に船があるかチェック（クエリで検索）
        let boat_at_dest = boat_query
            .iter()
            .find(|(_, pos)| pos.x == new_x && pos.y == new_y)
            .map(|(e, _)| e);

        if let Some(boat_entity) = boat_at_dest {
            // 船がある場所への移動 → 乗船
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (dx, dy),
            });
            commands.entity(entity).insert(OnBoat { boat_entity });
        } else {
            // 通常の徒歩移動
            match try_move(tile_pos.x, tile_pos.y, dx, dy, &map_data.grid) {
                MoveResult::Moved { new_x, new_y } => {
                    tile_pos.x = new_x;
                    tile_pos.y = new_y;
                    moved_events.write(PlayerMovedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                }
                MoveResult::Blocked => {
                    blocked_events.write(MovementBlockedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                }
            }
        }
    }
}

/// 船に乗っている間、船のTransformをプレイヤーに同期
pub fn sync_boat_with_player(
    player_query: Query<(&Transform, &OnBoat), With<Player>>,
    mut boat_query: Query<&mut Transform, (With<Boat>, Without<Player>)>,
) {
    let Ok((player_transform, on_boat)) = player_query.single() else {
        return;
    };

    if let Ok(mut boat_transform) = boat_query.get_mut(on_boat.boat_entity) {
        boat_transform.translation.x = player_transform.translation.x;
        boat_transform.translation.y = player_transform.translation.y;
    }
}
