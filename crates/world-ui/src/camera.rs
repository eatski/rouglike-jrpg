use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;

use movement_ui::Player;
use shared_ui::VISIBLE_SIZE;

use crate::map_mode::MapModeState;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: VISIBLE_SIZE,
                height: VISIBLE_SIZE,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

pub fn camera_follow(
    map_mode_state: Res<MapModeState>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    // マップモード時はプレイヤー追従しない
    if map_mode_state.enabled {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}
