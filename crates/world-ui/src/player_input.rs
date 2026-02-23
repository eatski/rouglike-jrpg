use bevy::prelude::*;

use world::wrap_position;

use movement_ui::{
    execute_move, process_movement_input, Boat, ExecuteMoveResult, MovementBlockedEvent,
    MovementLocked, OnBoat, PendingMove, Player, PlayerMovedEvent, TilePosition,
};
use app_state::FieldMenuOpen;
use movement_ui::{ActiveMap, MovementState};

use crate::map_mode::MapModeState;

/// プレイヤーの移動入力を処理するシステム
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn player_movement(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    active_map: Res<ActiveMap>,
    map_mode_state: Res<MapModeState>,
    field_menu_open: Option<Res<FieldMenuOpen>>,
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

    // ガード条件
    if locked.is_some() {
        return;
    }
    if map_mode_state.enabled {
        return;
    }
    if field_menu_open.is_some() {
        return;
    }

    let Some(input) = process_movement_input(&keyboard, &time, &mut move_state) else {
        return;
    };

    // 2回目の移動をPendingMoveとして予約するヘルパー
    let add_pending_move = |commands: &mut Commands, entity: Entity, direction: Option<(i32, i32)>| {
        if let Some(dir) = direction {
            commands.entity(entity).insert(PendingMove { direction: dir });
        }
    };

    if on_boat.is_none() {
        // === 徒歩: 乗船判定 ===
        let (new_x, new_y) = wrap_position(tile_pos.x, tile_pos.y, input.first_dx, input.first_dy);
        let boat_at_dest = boat_query
            .iter()
            .find(|(_, pos)| pos.x == new_x && pos.y == new_y)
            .map(|(e, _)| e);

        if let Some(boat_entity) = boat_at_dest {
            // 船がある場所への移動 → 乗船
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            add_pending_move(&mut commands, entity, input.pending_direction);
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (input.first_dx, input.first_dy),
            });
            commands.entity(entity).insert(OnBoat { boat_entity });
            return;
        }
    }

    // 通常移動（徒歩 or 船）
    if let ExecuteMoveResult::Success = execute_move(
        &mut commands, entity, &mut tile_pos, input.first_dx, input.first_dy,
        &active_map, on_boat, &mut boat_query,
        &mut moved_events, &mut blocked_events,
    ) {
        add_pending_move(&mut commands, entity, input.pending_direction);
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
