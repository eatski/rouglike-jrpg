use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

/// 自動スクリーンショットモードの設定
#[derive(Resource)]
pub struct AutoScreenshotMode {
    /// 撮影までの待機フレーム数
    pub delay_frames: u32,
    frame_count: u32,
    captured: bool,
}

impl AutoScreenshotMode {
    pub fn new(delay_frames: u32) -> Self {
        Self {
            delay_frames,
            frame_count: 0,
            captured: false,
        }
    }
}

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

/// 自動スクリーンショット＆終了（--screenshot モード）
pub fn auto_screenshot_system(
    mut commands: Commands,
    mut mode: ResMut<AutoScreenshotMode>,
    mut exit: MessageWriter<AppExit>,
) {
    mode.frame_count += 1;

    if !mode.captured && mode.frame_count >= mode.delay_frames {
        std::fs::create_dir_all("screenshots").ok();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("screenshots/latest.png"));
        mode.captured = true;
        info!("Auto screenshot captured: screenshots/latest.png");
    }

    // 撮影後、保存完了を待ってから終了
    if mode.captured && mode.frame_count >= mode.delay_frames + 30 {
        exit.write(AppExit::Success);
    }
}
