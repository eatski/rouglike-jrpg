use bevy::prelude::*;

use app_state::PartyState;
use battle_ui::{battle_display_system, setup_battle_scene};
use screenshot_common::{screenshot_app, setup_camera};

fn main() {
    let mut app = screenshot_app("battle");
    app.init_resource::<PartyState>()
        .add_systems(Startup, (setup_camera, setup_battle_scene).chain())
        .add_systems(Update, battle_display_system)
        .run();
}
