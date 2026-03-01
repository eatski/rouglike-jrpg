use bevy::prelude::*;

/// フィールド上のメッセージ表示状態（汎用）
#[derive(Resource, Default)]
pub struct FieldMessageState {
    pub message: Option<String>,
}

/// メッセージ表示用UIマーカー
#[derive(Component)]
pub struct FieldMessageUI;

/// メッセージ表示中でないことを判定する run_if 条件
pub fn field_message_not_active(
    state: Option<Res<FieldMessageState>>,
) -> bool {
    !state.is_some_and(|s| s.message.is_some())
}

/// メッセージ確認入力システム: 確認キーでメッセージをクリア
pub fn field_message_input_system(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut state: Option<ResMut<FieldMessageState>>,
) {
    let Some(ref mut state) = state else {
        return;
    };
    if state.message.is_none() {
        return;
    }
    if input_ui::is_confirm_just_pressed(&keyboard) || input_ui::is_cancel_just_pressed(&keyboard) {
        state.message = None;
        input_ui::clear_confirm_just_pressed(&mut keyboard);
    }
}

/// メッセージ表示UIシステム
pub fn field_message_display_system(
    mut commands: Commands,
    state: Option<Res<FieldMessageState>>,
    existing_ui: Query<Entity, With<FieldMessageUI>>,
) {
    let message = state.as_ref().and_then(|s| s.message.as_ref());

    match message {
        Some(msg) => {
            for entity in &existing_ui {
                commands.entity(entity).despawn();
            }

            commands
                .spawn((
                    FieldMessageUI,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(88.0),
                        left: Val::Px(16.0),
                        right: Val::Px(16.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                    GlobalZIndex(60),
                ))
                .with_child((
                    Text::new(msg.clone()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        }
        None => {
            for entity in &existing_ui {
                commands.entity(entity).despawn();
            }
        }
    }
}
