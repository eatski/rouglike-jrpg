use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};
use movement_ui::WINDOW_SIZE;

fn main() {
    let title = std::env::args()
        .nth(1)
        .map(|name| format!("Roguelike JRPG [{}]", name))
        .unwrap_or_else(|| "Roguelike JRPG".to_string());

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title,
                        resolution: WindowResolution::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            app_state::AppStatePlugin,
            movement_ui::MovementPlugin,
            world_ui::WorldPlugin,
            battle_ui::BattlePlugin,
            town_ui::TownPlugin,
            hokora_ui::HokoraPlugin,
            cave_ui::CavePlugin,
        ))
        .run();
}
