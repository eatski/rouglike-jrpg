use bevy::prelude::*;

use app_state::PartyState;
use screenshot_common::{screenshot_app, setup_camera};
use town_ui::{setup_town_scene, town_display_system};

fn main() {
    let mut app = screenshot_app("town");
    app.init_resource::<PartyState>()
        .add_systems(Startup, (setup_camera, setup_town_scene).chain())
        .add_systems(Update, town_display_system)
        .run();
}
