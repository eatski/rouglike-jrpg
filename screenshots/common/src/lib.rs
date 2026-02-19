use std::path::Path;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowResolution;

const OUTPUT_DIR: &str = "screenshots/output";

#[derive(Resource)]
struct ScreenshotState {
    frame: u32,
    output_path: String,
}

/// スクリーンショット用Appを構築する。
/// `name` はウィンドウタイトルと出力ファイル名（`screenshots/output/{name}.png`）に使われる。
pub fn screenshot_app(name: &str) -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: "../../assets".to_string(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("Screenshot: {}", name),
                    resolution: WindowResolution::new(768, 768),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(ScreenshotState {
        frame: 0,
        output_path: format!("{}/{}.png", OUTPUT_DIR, name),
    })
    .add_systems(Update, screenshot_system);
    app
}

/// UI専用シーン（battle/town）用のカメラ
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn screenshot_system(
    mut commands: Commands,
    mut state: ResMut<ScreenshotState>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frame += 1;
    if state.frame == 30 {
        if let Some(parent) = Path::new(&state.output_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(state.output_path.clone()));
    }
    if state.frame == 40 {
        exit.write(AppExit::Success);
    }
}
