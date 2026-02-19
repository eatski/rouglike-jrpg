use std::path::Path;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowResolution;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const OUTPUT_DIR: &str = "screenshots/output";
const DEFAULT_CAPTURE_FRAME: u32 = 30;
const DEFAULT_EXIT_FRAME: u32 = 40;

#[derive(Resource)]
struct ScreenshotState {
    frame: u32,
    output_path: String,
    capture_frame: u32,
    exit_frame: u32,
}

/// スクリーンショット用の固定シードRng
#[derive(Resource)]
pub struct ScreenshotRng {
    pub rng: ChaCha8Rng,
}

impl Default for ScreenshotRng {
    fn default() -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(42),
        }
    }
}

/// スクリーンショット用Appのビルダー
pub struct ScreenshotAppBuilder {
    name: String,
    window_width: u32,
    window_height: u32,
    capture_frame: u32,
    exit_frame: u32,
}

impl ScreenshotAppBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            window_width: 768,
            window_height: 768,
            capture_frame: DEFAULT_CAPTURE_FRAME,
            exit_frame: DEFAULT_EXIT_FRAME,
        }
    }

    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.window_width = width;
        self.window_height = height;
        self
    }

    pub fn capture_frame(mut self, frame: u32) -> Self {
        self.capture_frame = frame;
        self
    }

    pub fn output_path(&self) -> String {
        format!("{}/{}.png", OUTPUT_DIR, self.name)
    }

    pub fn build(self) -> App {
        let output_path = self.output_path();
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
                        title: format!("Screenshot: {}", self.name),
                        resolution: WindowResolution::new(self.window_width, self.window_height),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ScreenshotState {
            frame: 0,
            output_path,
            capture_frame: self.capture_frame,
            exit_frame: self.exit_frame,
        })
        .insert_resource(ScreenshotRng::default())
        .add_systems(Update, screenshot_system);
        app
    }
}

/// スクリーンショット用Appを構築する（後方互換ラッパー）。
/// `name` はウィンドウタイトルと出力ファイル名（`screenshots/output/{name}.png`）に使われる。
pub fn screenshot_app(name: &str) -> App {
    ScreenshotAppBuilder::new(name).build()
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
    if state.frame == state.capture_frame {
        if let Some(parent) = Path::new(&state.output_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(state.output_path.clone()));
    }
    if state.frame == state.exit_frame {
        exit.write(AppExit::Success);
    }
}
