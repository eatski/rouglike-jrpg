use bevy::prelude::*;

use animation_ui::Bounce;
use components_ui::{MovementLocked, PendingMove, Player, TilePosition};
use shared_ui::tile_to_world;
use shared_ui::MovementState;
use animation_ui::SmoothMove;

/// 町シーンのルートUIエンティティを識別するマーカー
#[derive(Component)]
pub struct TownSceneRoot;

/// 町メニューのフェーズ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TownMenuPhase {
    /// メニュー選択中
    MenuSelect,
    /// メッセージ表示中
    ShowMessage { message: String },
}

/// 町の状態管理リソース
#[derive(Resource)]
pub struct TownResource {
    /// 現在選択中のメニュー項目 (0=やどや, 1=話を聞く, 2=街を出る)
    pub selected_item: usize,
    /// 現在のフェーズ
    pub phase: TownMenuPhase,
}

/// 町メニュー項目のマーカー
#[derive(Component)]
pub struct TownMenuItem {
    pub index: usize,
}

/// 町メッセージテキストのマーカー
#[derive(Component)]
pub struct TownMessageText;

/// メッセージエリアの親ノードのマーカー
#[derive(Component)]
pub struct TownMessageArea;

const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
const UNSELECTED_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);

pub fn setup_town_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TownResource {
        selected_item: 0,
        phase: TownMenuPhase::MenuSelect,
    });

    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Bold.ttf");
    let panel_bg = Color::srgba(0.1, 0.1, 0.15, 0.85);
    let border_color = Color::srgb(0.4, 0.4, 0.5);

    commands
        .spawn((
            TownSceneRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            // タイトル
            parent.spawn((
                Text::new("まちに ついた"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(32.0)),
                    ..default()
                },
            ));

            // メニューパネル
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|menu| {
                    let items = ["> やどや", "  話を聞く", "  街を出る"];
                    for (i, label) in items.iter().enumerate() {
                        let color = if i == 0 {
                            SELECTED_COLOR
                        } else {
                            UNSELECTED_COLOR
                        };
                        menu.spawn((
                            TownMenuItem { index: i },
                            Text::new(*label),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                });

            // メッセージエリア（初期は非表示）
            parent
                .spawn((
                    TownMessageArea,
                    Node {
                        margin: UiRect::top(Val::Px(24.0)),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(300.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                    Visibility::Hidden,
                ))
                .with_children(|area| {
                    area.spawn((
                        TownMessageText,
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn cleanup_town_scene(
    mut commands: Commands,
    query: Query<Entity, With<TownSceneRoot>>,
    mut player_query: Query<(Entity, &TilePosition, &mut Transform), With<Player>>,
    mut move_state: ResMut<MovementState>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TownResource>();

    // プレイヤーの移動関連コンポーネントをクリーンアップ
    if let Ok((entity, tile_pos, mut transform)) = player_query.single_mut() {
        commands
            .entity(entity)
            .remove::<MovementLocked>()
            .remove::<SmoothMove>()
            .remove::<PendingMove>()
            .remove::<Bounce>();

        let (world_x, world_y) = tile_to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }

    *move_state = MovementState::default();
}

/// 町メニューの表示を更新するシステム
pub fn town_display_system(
    town_res: Res<TownResource>,
    mut menu_query: Query<
        (&TownMenuItem, &mut Text, &mut TextColor),
        (Without<TownMessageText>, Without<TownMessageArea>),
    >,
    mut message_query: Query<&mut Text, (With<TownMessageText>, Without<TownMessageArea>)>,
    mut message_area_query: Query<&mut Visibility, With<TownMessageArea>>,
) {
    let labels = ["やどや", "話を聞く", "街を出る"];

    for (item, mut text, mut color) in &mut menu_query {
        if item.index < labels.len() {
            let is_selected = item.index == town_res.selected_item;
            let prefix = if is_selected { "> " } else { "  " };
            **text = format!("{}{}", prefix, labels[item.index]);
            *color = if is_selected {
                TextColor(SELECTED_COLOR)
            } else {
                TextColor(UNSELECTED_COLOR)
            };
        }
    }

    let show_message = matches!(&town_res.phase, TownMenuPhase::ShowMessage { .. });

    for mut vis in &mut message_area_query {
        *vis = if show_message {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut text in &mut message_query {
        if let TownMenuPhase::ShowMessage { message } = &town_res.phase {
            **text = message.clone();
        }
    }
}
