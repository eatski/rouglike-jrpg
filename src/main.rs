use std::time::Duration;

use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};
use bevy::winit::{UpdateMode, WinitSettings};
use app_state::InField;
use field_core::WINDOW_SIZE;

fn main() {
    let title = std::env::args()
        .nth(1)
        .map(|name| format!("Roguelike JRPG [{}]", name))
        .unwrap_or_else(|| "Roguelike JRPG".to_string());

    App::new()
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive(Duration::from_millis(16)),
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_millis(100)),
        })
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
            field_walk_ui::MovementPlugin,
            world_ui::WorldPlugin,
            battle_ui::BattlePlugin,
            town_ui::TownPlugin,
            hokora_ui::HokoraPlugin,
            cave_ui::CavePlugin,
        ))
        .add_systems(OnExit(InField), field_walk_ui::cleanup_player_movement)
        .run();
}
