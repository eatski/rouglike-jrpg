use bevy::prelude::*;

use movement_ui::{Bounce, MovementLocked, PendingMove, Player, SmoothMove, TilePosition};
use party::shop_items;
use shared_ui::{ActiveMap, MovementState, PartyState};

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
    /// 道具屋 — アイテム選択中
    ShopSelect { selected: usize },
    /// 道具屋 — 購入結果メッセージ表示中
    ShopMessage { message: String },
}

/// 町の状態管理リソース
#[derive(Resource)]
pub struct TownResource {
    /// 現在選択中のメニュー項目 (0=やどや, 1=道具屋, 2=話を聞く, 3=街を出る)
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

/// メインメニューパネルのマーカー
#[derive(Component)]
pub struct TownMainMenu;

/// ショップパネルのルートマーカー
#[derive(Component)]
pub struct ShopMenuRoot;

/// ショップメニュー項目のマーカー
#[derive(Component)]
pub struct ShopMenuItem {
    pub index: usize,
}

/// ショップゴールド表示のマーカー
#[derive(Component)]
pub struct ShopGoldText;

const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
const UNSELECTED_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);

pub fn setup_town_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    party_state: Res<PartyState>,
) {
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

            // メインメニューパネル
            parent
                .spawn((
                    TownMainMenu,
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
                    let items = ["> やどや", "  道具屋", "  話を聞く", "  街を出る"];
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

            // ショップパネル（初期は非表示）
            parent
                .spawn((
                    ShopMenuRoot,
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(8.0),
                        min_width: Val::Px(250.0),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                    Visibility::Hidden,
                ))
                .with_children(|shop| {
                    // ゴールド表示
                    shop.spawn((
                        ShopGoldText,
                        Text::new(format!("所持金: {}G", party_state.gold)),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.3)),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    // ショップアイテム一覧
                    for (i, item) in shop_items().iter().enumerate() {
                        let label = format!(
                            "{}{}  {}G",
                            if i == 0 { "> " } else { "  " },
                            item.name(),
                            item.price()
                        );
                        let color = if i == 0 {
                            SELECTED_COLOR
                        } else {
                            UNSELECTED_COLOR
                        };
                        shop.spawn((
                            ShopMenuItem { index: i },
                            Text::new(label),
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
    active_map: Res<ActiveMap>,
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

        let (world_x, world_y) = active_map.to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }

    *move_state = MovementState::default();
}

/// 町メニューの表示を更新するシステム
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn town_display_system(
    town_res: Res<TownResource>,
    party_state: Res<PartyState>,
    mut main_menu_query: Query<&mut Visibility, (With<TownMainMenu>, Without<ShopMenuRoot>, Without<TownMessageArea>)>,
    mut shop_root_query: Query<&mut Visibility, (With<ShopMenuRoot>, Without<TownMainMenu>, Without<TownMessageArea>)>,
    mut menu_query: Query<
        (&TownMenuItem, &mut Text, &mut TextColor),
        (Without<TownMessageText>, Without<TownMessageArea>, Without<ShopMenuItem>),
    >,
    mut shop_item_query: Query<
        (&ShopMenuItem, &mut Text, &mut TextColor),
        (Without<TownMenuItem>, Without<TownMessageText>, Without<TownMessageArea>),
    >,
    mut gold_query: Query<&mut Text, (With<ShopGoldText>, Without<TownMenuItem>, Without<ShopMenuItem>, Without<TownMessageText>, Without<TownMessageArea>)>,
    mut message_query: Query<&mut Text, (With<TownMessageText>, Without<TownMessageArea>, Without<TownMenuItem>, Without<ShopMenuItem>, Without<ShopGoldText>)>,
    mut message_area_query: Query<&mut Visibility, (With<TownMessageArea>, Without<TownMainMenu>, Without<ShopMenuRoot>)>,
) {
    let in_shop = matches!(
        &town_res.phase,
        TownMenuPhase::ShopSelect { .. } | TownMenuPhase::ShopMessage { .. }
    );

    // メインメニュー表示/非表示
    for mut vis in &mut main_menu_query {
        *vis = if in_shop {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    // ショップパネル表示/非表示
    for mut vis in &mut shop_root_query {
        *vis = if in_shop {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // メインメニュー項目の更新
    let labels = ["やどや", "道具屋", "話を聞く", "街を出る"];
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

    // ショップアイテムの更新
    if let TownMenuPhase::ShopSelect { selected } = &town_res.phase {
        let items = shop_items();
        for (shop_item, mut text, mut color) in &mut shop_item_query {
            if shop_item.index < items.len() {
                let is_selected = shop_item.index == *selected;
                let prefix = if is_selected { "> " } else { "  " };
                **text = format!("{}{}  {}G", prefix, items[shop_item.index].name(), items[shop_item.index].price());
                *color = if is_selected {
                    TextColor(SELECTED_COLOR)
                } else {
                    TextColor(UNSELECTED_COLOR)
                };
            }
        }
    }

    // ゴールド表示更新
    for mut text in &mut gold_query {
        **text = format!("所持金: {}G", party_state.gold);
    }

    // メッセージエリアの表示/非表示
    let show_message = matches!(
        &town_res.phase,
        TownMenuPhase::ShowMessage { .. } | TownMenuPhase::ShopMessage { .. }
    );

    for mut vis in &mut message_area_query {
        *vis = if show_message {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut text in &mut message_query {
        match &town_res.phase {
            TownMenuPhase::ShowMessage { message } | TownMenuPhase::ShopMessage { message } => {
                **text = message.clone();
            }
            _ => {}
        }
    }
}
