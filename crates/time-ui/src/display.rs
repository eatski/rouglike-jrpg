use bevy::prelude::*;

use movement_ui::TileEnteredEvent;

use crate::resources::TimeCounter;

/// 時間表示UIルートのマーカー（cleanup用）
#[derive(Component)]
pub struct TimeDisplayRoot;

/// 時間表示テキストのマーカー（テキスト更新用）
#[derive(Component)]
pub struct TimeDisplayText;

/// Exploring状態開始時に右上に時間表示UIをspawnするシステム
pub fn setup_time_display(mut commands: Commands, asset_server: Res<AssetServer>, time_counter: Res<TimeCounter>) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Bold.ttf");

    commands
        .spawn((
            TimeDisplayRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.6)),
            GlobalZIndex(50),
        ))
        .with_children(|parent| {
            parent.spawn((
                TimeDisplayText,
                Text::new(format!("{} pt", time_counter.time_count.count())),
                TextFont {
                    font,
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// TileEnteredEvent受信時にTimeCounterを+1するシステム
pub fn update_time_counter(
    mut time_counter: ResMut<TimeCounter>,
    mut events: MessageReader<TileEnteredEvent>,
) {
    for _event in events.read() {
        time_counter.time_count.increment();
    }
}

/// TimeCounter変更時にテキストを更新するシステム
pub fn update_time_display(
    time_counter: Res<TimeCounter>,
    mut query: Query<&mut Text, With<TimeDisplayText>>,
) {
    if !time_counter.is_changed() {
        return;
    }

    for mut text in &mut query {
        **text = format!("{} pt", time_counter.time_count.count());
    }
}

/// MapMode時に時間表示を非表示にするシステム
pub fn toggle_time_display_visibility(
    map_mode: Res<world_ui::MapModeState>,
    mut query: Query<&mut Visibility, With<TimeDisplayRoot>>,
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

/// Exploring状態終了時に時間表示UIをdespawnするシステム
pub fn cleanup_time_display(mut commands: Commands, query: Query<Entity, With<TimeDisplayRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
