use bevy::prelude::*;

use app_state::PartyState;

use crate::map_mode::MapModeState;

/// HUDルートエンティティのマーカー
#[derive(Component)]
pub struct HudRoot;

/// HUD上のHP表示テキスト
#[derive(Component)]
pub struct HudHpText {
    pub index: usize,
}

/// HUD上のMP表示テキスト
#[derive(Component)]
pub struct HudMpText {
    pub index: usize,
}

/// HUD上のHPバー前景
#[derive(Component)]
pub struct HudHpBarFill {
    pub index: usize,
}

/// HUD上の名前+レベル表示テキスト
#[derive(Component)]
pub struct HudNameText {
    pub index: usize,
}

/// HUD上の所持金テキスト
#[derive(Component)]
pub struct HudGoldText;

/// HP割合に応じた色を返す（>50%=緑, >25%=黄, ≤25%=赤）
fn hp_bar_color(ratio: f32) -> Color {
    if ratio > 0.5 {
        Color::srgb(0.2, 0.8, 0.2)
    } else if ratio > 0.25 {
        Color::srgb(0.9, 0.8, 0.1)
    } else {
        Color::srgb(0.9, 0.2, 0.2)
    }
}

/// Exploring状態開始時にHUDをspawnするシステム
pub fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>, party_state: Res<PartyState>) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Bold.ttf");
    let panel_bg = Color::srgba(0.1, 0.1, 0.15, 0.6);
    let hp_bar_bg = Color::srgb(0.2, 0.2, 0.2);

    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(80.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(panel_bg),
            GlobalZIndex(50),
        ))
        .with_children(|panel| {
            for (i, member) in party_state.members.iter().enumerate() {
                let name = member.kind.name();
                let ratio = member.stats.hp as f32 / member.stats.max_hp as f32;

                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(2.0),
                        ..default()
                    })
                    .with_children(|col| {
                        // 名前+レベル
                        col.spawn((
                            HudNameText { index: i },
                            Text::new(format!("{} Lv.{}", name, member.level)),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // HP/MPを横並びにする行
                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|row| {
                            // HPテキスト
                            row.spawn((
                                HudHpText { index: i },
                                Text::new(format!("{}/{}", member.stats.hp, member.stats.max_hp)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            // MPテキスト
                            row.spawn((
                                HudMpText { index: i },
                                Text::new(format!("{}/{}", member.stats.mp, member.stats.max_mp)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.6, 0.8, 1.0)),
                            ));
                        });

                        // HPバー
                        col.spawn((
                            Node {
                                width: Val::Px(70.0),
                                height: Val::Px(6.0),
                                ..default()
                            },
                            BackgroundColor(hp_bar_bg),
                        ))
                        .with_children(|bar_bg| {
                            bar_bg.spawn((
                                HudHpBarFill { index: i },
                                Node {
                                    width: Val::Percent(ratio * 100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(hp_bar_color(ratio)),
                            ));
                        });
                    });
            }

            // 所持金
            panel.spawn((
                HudGoldText,
                Text::new(format!("{}G", party_state.gold)),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
            ));
        });
}

type GoldTextQuery<'w, 's> = Query<'w, 's, &'static mut Text, (With<HudGoldText>, Without<HudHpText>, Without<HudMpText>, Without<HudNameText>)>;

/// PartyState変化時にHUDのHP/MP/バー/名前+レベルを更新するシステム
pub fn update_hud(
    party_state: Res<PartyState>,
    mut name_query: Query<(&HudNameText, &mut Text), (Without<HudHpText>, Without<HudMpText>, Without<HudGoldText>)>,
    mut hp_query: Query<(&HudHpText, &mut Text), (Without<HudNameText>, Without<HudMpText>, Without<HudGoldText>)>,
    mut mp_query: Query<(&HudMpText, &mut Text), (Without<HudNameText>, Without<HudHpText>, Without<HudGoldText>)>,
    mut bar_query: Query<(&HudHpBarFill, &mut Node, &mut BackgroundColor)>,
    mut gold_query: GoldTextQuery,
) {
    if !party_state.is_changed() {
        return;
    }

    for (name_text, mut text) in &mut name_query {
        if let Some(member) = party_state.members.get(name_text.index) {
            **text = format!("{} Lv.{}", member.kind.name(), member.level);
        }
    }

    for (hp_text, mut text) in &mut hp_query {
        if let Some(member) = party_state.members.get(hp_text.index) {
            **text = format!("{}/{}", member.stats.hp, member.stats.max_hp);
        }
    }

    for (mp_text, mut text) in &mut mp_query {
        if let Some(member) = party_state.members.get(mp_text.index) {
            **text = format!("{}/{}", member.stats.mp, member.stats.max_mp);
        }
    }

    for (bar, mut node, mut bg) in &mut bar_query {
        if let Some(member) = party_state.members.get(bar.index) {
            let ratio = member.stats.hp as f32 / member.stats.max_hp as f32;
            node.width = Val::Percent(ratio * 100.0);
            *bg = BackgroundColor(hp_bar_color(ratio));
        }
    }

    for mut text in &mut gold_query {
        **text = format!("{}G", party_state.gold);
    }
}

/// Exploring状態終了時にHUDをdespawnするシステム
pub fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// MapModeState変化時にHUDの表示/非表示を切り替えるシステム
pub fn toggle_hud_visibility(
    map_mode: Res<MapModeState>,
    mut query: Query<&mut Visibility, With<HudRoot>>,
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
