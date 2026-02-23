use bevy::prelude::*;

use app_state::HokoraPositions;
use movement_ui::{Bounce, MovementLocked, PendingMove, Player, SmoothMove, TilePosition};
use movement_ui::MovementState;

/// 祠シーンのルートUIエンティティを識別するマーカー
#[derive(Component)]
pub struct HokoraSceneRoot;

/// 祠メニューのフェーズ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HokoraMenuPhase {
    /// メニュー選択中
    MenuSelect,
    /// メッセージ表示中
    ShowMessage { message: String },
}

/// 祠の状態管理リソース
#[derive(Resource)]
pub struct HokoraResource {
    /// 現在選択中のメニュー項目 (0=様子を見る, 1=扉を開ける, 2=出る)
    pub selected_item: usize,
    /// 現在のフェーズ
    pub phase: HokoraMenuPhase,
    /// ワープ先の座標（もう一方の祠）
    pub warp_destination: Option<(usize, usize)>,
    /// 最寄り祠のインデックス（0始まり、必要月のかけら数の算出に使用）
    pub hokora_index: usize,
    /// ワープ済みフラグ（メッセージ後にフィールドへ遷移するため）
    pub warped: bool,
}

/// メニュー項目のマーカー
#[derive(Component)]
pub struct HokoraMenuItem {
    pub index: usize,
}

/// メッセージテキストのマーカー
#[derive(Component)]
pub struct HokoraMessageText;

/// メッセージエリアの親ノードのマーカー
#[derive(Component)]
pub struct HokoraMessageArea;

const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
const UNSELECTED_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);

pub fn setup_hokora_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hokora_positions: Res<HokoraPositions>,
    player_query: Query<&TilePosition, With<Player>>,
) {
    // 最寄りの祠を特定し、対応するワープ先とインデックスを取得
    let (warp_destination, hokora_index) = if let Ok(pos) = player_query.single() {
        let idx = hokora_positions.nearest(pos.x, pos.y);
        let dest = hokora_positions.warp_destination(idx);
        (dest, idx)
    } else {
        (None, 0)
    };

    commands.insert_resource(HokoraResource {
        selected_item: 0,
        phase: HokoraMenuPhase::MenuSelect,
        warp_destination,
        hokora_index,
        warped: false,
    });

    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Bold.ttf");
    let panel_bg = Color::srgba(0.1, 0.1, 0.15, 0.85);
    let border_color = Color::srgb(0.4, 0.4, 0.5);

    commands
        .spawn((
            HokoraSceneRoot,
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
                Text::new("ほこらに ついた"),
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
                    let items = ["> 様子を見る", "  扉を開ける", "  出る"];
                    for (i, label) in items.iter().enumerate() {
                        let color = if i == 0 {
                            SELECTED_COLOR
                        } else {
                            UNSELECTED_COLOR
                        };
                        menu.spawn((
                            HokoraMenuItem { index: i },
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
                    HokoraMessageArea,
                    Node {
                        margin: UiRect::top(Val::Px(24.0)),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(300.0),
                        justify_content: JustifyContent::Center,
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|area| {
                    area.spawn((
                        HokoraMessageText,
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

pub fn cleanup_hokora_scene(
    mut commands: Commands,
    query: Query<Entity, With<HokoraSceneRoot>>,
    player_query: Query<Entity, With<Player>>,
    mut move_state: ResMut<MovementState>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<HokoraResource>();

    // プレイヤーの移動関連コンポーネントをクリーンアップ
    if let Ok(entity) = player_query.single() {
        commands
            .entity(entity)
            .remove::<MovementLocked>()
            .remove::<SmoothMove>()
            .remove::<PendingMove>()
            .remove::<Bounce>();
    }

    *move_state = MovementState::default();
}

/// 祠メニューの表示を更新するシステム
pub fn hokora_display_system(
    hokora_res: Res<HokoraResource>,
    mut menu_query: Query<
        (&HokoraMenuItem, &mut Text, &mut TextColor),
        (Without<HokoraMessageText>, Without<HokoraMessageArea>),
    >,
    mut message_query: Query<
        &mut Text,
        (
            With<HokoraMessageText>,
            Without<HokoraMenuItem>,
            Without<HokoraMessageArea>,
        ),
    >,
    mut message_area_query: Query<&mut Node, With<HokoraMessageArea>>,
) {
    // メニュー項目の更新
    let labels = ["様子を見る", "扉を開ける", "出る"];
    for (item, mut text, mut color) in &mut menu_query {
        if item.index < labels.len() {
            let is_selected = item.index == hokora_res.selected_item;
            let prefix = if is_selected { "> " } else { "  " };
            **text = format!("{}{}", prefix, labels[item.index]);
            *color = if is_selected {
                TextColor(SELECTED_COLOR)
            } else {
                TextColor(UNSELECTED_COLOR)
            };
        }
    }

    // メッセージエリアの表示/非表示
    let show_message = matches!(&hokora_res.phase, HokoraMenuPhase::ShowMessage { .. });
    for mut node in &mut message_area_query {
        node.display = if show_message {
            Display::Flex
        } else {
            Display::None
        };
    }

    // メッセージテキストの更新
    for mut text in &mut message_query {
        if let HokoraMenuPhase::ShowMessage { message } = &hokora_res.phase {
            **text = message.clone();
        }
    }
}
