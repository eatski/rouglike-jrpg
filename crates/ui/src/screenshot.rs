use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

/// F12キーでスクリーンショットを手動撮影
pub fn manual_screenshot_system(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut counter: Local<u32>,
) {
    if input.just_pressed(KeyCode::F12) {
        std::fs::create_dir_all("screenshots").ok();
        let path = format!("screenshots/screenshot-{}.png", *counter);
        *counter += 1;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
        info!("Screenshot saved: {}", path);
    }
}
