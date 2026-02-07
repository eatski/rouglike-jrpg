use bevy::prelude::*;

use game::battle::{BattleState, Enemy};

use crate::bounce::Bounce;
use crate::components::{MovementLocked, PendingMove, Player, TilePosition};
use crate::constants::tile_to_world;
use crate::resources::MovementState;
use crate::smooth_move::SmoothMove;

use super::display::{
    CommandCursor, EnemyHpBarFill, EnemyHpText, MessageText, PlayerHpBarFill, PlayerHpText,
};

/// 戦闘シーンのルートUIエンティティを識別するマーカー
#[derive(Component)]
pub struct BattleSceneRoot;

/// 戦闘の状態管理リソース
#[derive(Resource)]
pub struct BattleResource {
    pub state: BattleState,
    /// 現在選択中のコマンドインデックス (0=たたかう, 1=にげる)
    pub selected_command: usize,
    /// 戦闘フェーズ
    pub phase: BattlePhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattlePhase {
    /// コマンド選択中
    CommandSelect,
    /// メッセージ表示中（Enterで次へ）
    ShowMessage { messages: Vec<String>, index: usize },
    /// 戦闘終了（Enterでフィールドに戻る）
    BattleOver { message: String },
}

/// 敵スプライトのマーカー
#[derive(Component)]
pub struct EnemySprite;

pub fn setup_battle_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let enemy = Enemy::slime();
    let enemy_name = enemy.kind.name().to_string();
    let enemy_max_hp = enemy.stats.max_hp;
    let battle_state = BattleState::new(enemy);
    let player_max_hp = battle_state.player.max_hp;

    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Bold.ttf");

    commands.insert_resource(BattleResource {
        state: battle_state,
        selected_command: 0,
        phase: BattlePhase::ShowMessage {
            messages: vec![format!("{}が あらわれた！", enemy_name)],
            index: 0,
        },
    });

    let panel_bg = Color::srgba(0.1, 0.1, 0.15, 0.85);
    let border_color = Color::srgb(0.4, 0.4, 0.5);
    let hp_bar_bg = Color::srgb(0.2, 0.2, 0.2);
    let hp_bar_green = Color::srgb(0.2, 0.8, 0.2);
    let selected_color = Color::srgb(1.0, 0.9, 0.2);
    let unselected_color = Color::srgb(0.6, 0.6, 0.6);

    // 全画面を覆う黒背景UI
    commands
        .spawn((
            BattleSceneRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            // === 上部 (40%): 敵表示エリア ===
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(40.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|area| {
                    // 敵スプライト (128px)
                    area.spawn((
                        EnemySprite,
                        ImageNode::new(asset_server.load("enemies/slime.png")),
                        Node {
                            width: Val::Px(128.0),
                            height: Val::Px(128.0),
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    // 敵名前とHP
                    area.spawn((
                        EnemyHpText,
                        Text::new(format!(
                            "{} HP: {}/{}",
                            enemy_name, enemy_max_hp, enemy_max_hp
                        )),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                    ));

                    // 敵HPバー（背景 + 前景）
                    area.spawn((
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(8.0),
                            ..default()
                        },
                        BackgroundColor(hp_bar_bg),
                    ))
                    .with_children(|bar_bg| {
                        bar_bg.spawn((
                            EnemyHpBarFill,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(hp_bar_green),
                        ));
                    });
                });

            // === 中部 (30%): メッセージエリア ===
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|area| {
                    area.spawn((
                        MessageText,
                        Text::new(format!("{}が あらわれた！", enemy_name)),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // === 下部 (30%): ステータス＋コマンド ===
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(30.0),
                        flex_direction: FlexDirection::Row,
                        padding: UiRect::all(Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|area| {
                    // 左: プレイヤーステータス
                    area.spawn(Node {
                        width: Val::Percent(50.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|hp_area| {
                        // 「勇者」ラベル
                        hp_area.spawn((
                            Text::new("勇者"),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            },
                        ));

                        // HP テキスト
                        hp_area.spawn((
                            PlayerHpText,
                            Text::new(format!("HP: {}/{}", player_max_hp, player_max_hp)),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            },
                        ));

                        // プレイヤーHPバー（背景 + 前景）
                        hp_area
                            .spawn((
                                Node {
                                    width: Val::Px(100.0),
                                    height: Val::Px(8.0),
                                    ..default()
                                },
                                BackgroundColor(hp_bar_bg),
                            ))
                            .with_children(|bar_bg| {
                                bar_bg.spawn((
                                    PlayerHpBarFill,
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(hp_bar_green),
                                ));
                            });
                    });

                    // 右: コマンド
                    area.spawn(Node {
                        width: Val::Percent(50.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::left(Val::Px(16.0)),
                        ..default()
                    })
                    .with_children(|cmd_area| {
                        cmd_area.spawn((
                            CommandCursor { index: 0 },
                            Text::new("> たたかう"),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(selected_color),
                        ));
                        cmd_area.spawn((
                            CommandCursor { index: 1 },
                            Text::new("  にげる"),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(unselected_color),
                        ));
                    });
                });
        });
}

pub fn cleanup_battle_scene(
    mut commands: Commands,
    query: Query<Entity, With<BattleSceneRoot>>,
    mut player_query: Query<(Entity, &TilePosition, &mut Transform), With<Player>>,
    mut move_state: ResMut<MovementState>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<BattleResource>();

    // プレイヤーの移動関連コンポーネントをクリーンアップ
    if let Ok((entity, tile_pos, mut transform)) = player_query.single_mut() {
        commands
            .entity(entity)
            .remove::<MovementLocked>()
            .remove::<SmoothMove>()
            .remove::<PendingMove>()
            .remove::<Bounce>();

        // タイル座標に基づいて正確な位置にスナップ
        let (world_x, world_y) = tile_to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }

    // 移動状態をリセット
    *move_state = MovementState::default();
}
