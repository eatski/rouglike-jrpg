use bevy::prelude::*;

use crate::map_mode::MapModeState;
use hud_ui::HudRoot;

/// MapModeState変化時にHUDの表示/非表示を切り替えるシステム
pub fn toggle_hud_visibility(
    map_mode: Res<MapModeState>,
    mut query: Query<&mut Visibility, With<HudRoot>>,
) {
    if !map_mode.is_changed() {
        return;
    }

    for mut vis in &mut query {
        *vis = if map_mode.enabled {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}
