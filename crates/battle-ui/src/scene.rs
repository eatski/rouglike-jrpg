use bevy::prelude::*;

use battle::{generate_enemy_group, BattleAction, BattleState, EnemyKind, ItemKind, SpellKind};

use movement_ui::{Bounce, MovementLocked, PendingMove, Player, TilePosition};
use shared_ui::{ActiveMap, MovementState, PartyState};

use super::display::{
    CommandCursor, EnemyNameLabel, MessageText, PartyMemberHpBarFill, PartyMemberHpText,
    PartyMemberMpText, PartyMemberNameText, TargetCursor,
};

/// 戦闘シーンのルートUIエンティティを識別するマーカー
#[derive(Component)]
pub struct BattleSceneRoot;

/// パーティ全員分のコマンド蓄積（メンバーインデックスで管理）
#[derive(Debug, Clone)]
pub struct PendingCommands {
    slots: Vec<Option<BattleAction>>,
}

impl PendingCommands {
    pub fn new(party_size: usize) -> Self {
        Self {
            slots: vec![None; party_size],
        }
    }

    pub fn set(&mut self, member_index: usize, action: BattleAction) {
        if member_index < self.slots.len() {
            self.slots[member_index] = Some(action);
        }
    }

    pub fn remove(&mut self, member_index: usize) {
        if member_index < self.slots.len() {
            self.slots[member_index] = None;
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }

    /// パーティメンバーインデックスで正しく並んだコマンドVecを返す
    pub fn to_commands(&self) -> Vec<BattleAction> {
        self.slots
            .iter()
            .map(|opt| {
                opt.unwrap_or(BattleAction::Attack {
                    target: battle::TargetId::Enemy(0),
                })
            })
            .collect()
    }
}

/// メッセージ表示時に適用する視覚効果
#[derive(Debug, Clone)]
pub enum MessageEffect {
    /// パーティメンバーのHP表示値を更新
    UpdatePartyHp { member_index: usize, new_hp: i32 },
    /// パーティメンバーのMP表示値を更新
    UpdatePartyMp { member_index: usize, new_mp: i32 },
    /// 敵を非表示にする
    HideEnemy { enemy_index: usize },
    /// 画面シェイク
    Shake,
    /// 敵スプライト点滅
    BlinkEnemy { enemy_index: usize },
}

/// 戦闘のゲームロジック状態（game crateのBattleStateをラップ）
#[derive(Resource)]
pub struct BattleGameState {
    pub state: BattleState,
}

/// 戦闘のUI状態管理リソース
#[derive(Resource)]
pub struct BattleUIState {
    /// 現在選択中のコマンドインデックス (0=たたかう, 1=じゅもん, 2=にげる)
    pub selected_command: usize,
    /// ターゲット選択中の敵インデックス
    pub selected_target: usize,
    /// パーティ全員分のコマンド蓄積
    pub pending_commands: PendingCommands,
    /// 戦闘フェーズ
    pub phase: BattlePhase,
    /// 敵ごとの視覚的非表示フラグ（「たおした」メッセージ表示時にtrueになる）
    pub hidden_enemies: Vec<bool>,
    /// 表示用パーティHP（メッセージ送りに連動して更新）
    pub display_party_hp: Vec<i32>,
    /// 表示用パーティMP（メッセージ送りに連動して更新）
    pub display_party_mp: Vec<i32>,
    /// 呪文選択中のカーソル位置
    pub selected_spell: usize,
    /// 選択済みの呪文（ターゲット選択へ渡す）
    pub pending_spell: Option<SpellKind>,
    /// アイテム選択中のカーソル位置
    pub selected_item: usize,
    /// 選択済みのアイテム（ターゲット選択へ渡す）
    pub pending_item: Option<ItemKind>,
    /// 味方ターゲット選択中のカーソル位置
    pub selected_ally_target: usize,
    /// メッセージindex → 適用する視覚効果のリスト
    pub message_effects: Vec<(usize, MessageEffect)>,
    /// 画面シェイク用タイマー
    pub shake_timer: Option<Timer>,
    /// 敵スプライト点滅用タイマー
    pub blink_timer: Option<Timer>,
    /// 点滅中の敵インデックス
    pub blink_enemy: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattlePhase {
    /// コマンド選択中（member_indexで誰のコマンドか）
    CommandSelect { member_index: usize },
    /// 呪文選択中
    SpellSelect { member_index: usize },
    /// アイテム選択中
    ItemSelect { member_index: usize },
    /// ターゲット選択中（敵）
    TargetSelect { member_index: usize },
    /// 味方ターゲット選択中（回復呪文用）
    AllyTargetSelect { member_index: usize },
    /// メッセージ表示中（Enterで次へ）
    ShowMessage { messages: Vec<String>, index: usize },
    /// 戦闘終了（Enterでフィールドに戻る）
    BattleOver { message: String },
}

/// 敵スプライトのマーカー
#[derive(Component)]
pub struct EnemySprite {
    pub index: usize,
}

/// 同種の敵にサフィックスを付与した表示名を生成
fn enemy_display_names(enemies: &[battle::Enemy]) -> Vec<String> {
    // 同種の敵が複数いる場合のみサフィックス付与
    let mut kind_counts: std::collections::HashMap<EnemyKind, usize> =
        std::collections::HashMap::new();
    for e in enemies {
        *kind_counts.entry(e.kind).or_insert(0) += 1;
    }

    let suffixes = ['A', 'B', 'C', 'D'];
    let mut kind_indices: std::collections::HashMap<EnemyKind, usize> =
        std::collections::HashMap::new();

    enemies
        .iter()
        .map(|e| {
            let count = kind_counts[&e.kind];
            if count > 1 {
                let idx = kind_indices.entry(e.kind).or_insert(0);
                let suffix = suffixes.get(*idx).unwrap_or(&'?');
                *idx += 1;
                format!("{}{}", e.kind.name(), suffix)
            } else {
                e.kind.name().to_string()
            }
        })
        .collect()
}

pub fn setup_battle_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    party_state: Res<PartyState>,
) {
    let party = party_state.members.clone();
    let enemies = generate_enemy_group(rand::random::<f32>(), rand::random::<f32>());
    let display_names = enemy_display_names(&enemies);

    let encounter_msg = if enemies.len() == 1 {
        format!("{}が あらわれた！", display_names[0])
    } else {
        format!(
            "{}が {}匹 あらわれた！",
            enemies[0].kind.name(),
            enemies.len()
        )
    };

    let enemy_count = enemies.len();
    // スプライトハンドルを事前にロード（enemiesはBattleState::newでmoveされるため）
    let enemy_sprite_handles: Vec<Handle<Image>> = enemies
        .iter()
        .map(|e| asset_server.load(e.kind.sprite_path()))
        .collect();
    let battle_state = BattleState::new(party, enemies, party_state.inventory.clone());

    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Bold.ttf");

    let party_size = battle_state.party.len();
    let display_party_hp = battle_state.party.iter().map(|m| m.stats.hp).collect();
    let display_party_mp = battle_state.party.iter().map(|m| m.stats.mp).collect();

    commands.insert_resource(BattleGameState {
        state: battle_state,
    });
    commands.insert_resource(BattleUIState {
        selected_command: 0,
        selected_target: 0,
        pending_commands: PendingCommands::new(party_size),
        phase: BattlePhase::ShowMessage {
            messages: vec![encounter_msg],
            index: 0,
        },
        hidden_enemies: vec![false; enemy_count],
        display_party_hp,
        display_party_mp,
        selected_spell: 0,
        pending_spell: None,
        selected_item: 0,
        pending_item: None,
        selected_ally_target: 0,
        message_effects: Vec::new(),
        shake_timer: None,
        blink_timer: None,
        blink_enemy: None,
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
            build_enemy_area(parent, &font, &asset_server, &enemy_sprite_handles);

            // === 中部 (30%): メッセージエリア ===
            build_message_area(parent, &font, panel_bg, border_color);

            // === 下部 (30%): パーティステータス＋コマンド ===
            build_bottom_area(
                parent,
                &font,
                panel_bg,
                border_color,
                hp_bar_bg,
                hp_bar_green,
                selected_color,
                unselected_color,
            );
        });
}

fn build_enemy_area(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    asset_server: &AssetServer,
    enemy_sprite_handles: &[Handle<Image>],
) {
    let default_handle = asset_server.load("enemies/slime.png");
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(40.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|area| {
            // 最大4匹分のスロットを構築（非表示のものも含む）
            for i in 0..4 {
                let sprite_handle = enemy_sprite_handles
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| default_handle.clone());

                area.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|slot| {
                    // ターゲットカーソル(▼)
                    slot.spawn((
                        TargetCursor { index: i },
                        Text::new("▼"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.2)),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                        Visibility::Hidden,
                    ));

                    // 敵スプライト
                    slot.spawn((
                        EnemySprite { index: i },
                        ImageNode::new(sprite_handle.clone()),
                        Node {
                            width: Val::Px(96.0),
                            height: Val::Px(96.0),
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                        Visibility::Hidden,
                    ));

                    // 敵名ラベル
                    slot.spawn((
                        EnemyNameLabel { index: i },
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(2.0)),
                            ..default()
                        },
                        Visibility::Hidden,
                    ));
                });
            }
        });
}

fn build_message_area(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    panel_bg: Color,
    border_color: Color,
) {
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
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

#[allow(clippy::too_many_arguments)]
fn build_bottom_area(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    panel_bg: Color,
    border_color: Color,
    hp_bar_bg: Color,
    hp_bar_green: Color,
    selected_color: Color,
    unselected_color: Color,
) {
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
            // 左: パーティステータス（横並び）
            area.spawn(Node {
                width: Val::Percent(60.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|party_area| {
                let member_names = ["勇者", "魔法使い", "僧侶"];
                for (i, name) in member_names.iter().enumerate() {
                    party_area
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|member_col| {
                            // 名前
                            member_col.spawn((
                                PartyMemberNameText { index: i },
                                Text::new(*name),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    margin: UiRect::bottom(Val::Px(2.0)),
                                    ..default()
                                },
                            ));

                            // HPテキスト
                            member_col.spawn((
                                PartyMemberHpText { index: i },
                                Text::new(""),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    margin: UiRect::bottom(Val::Px(2.0)),
                                    ..default()
                                },
                            ));

                            // MPテキスト
                            member_col.spawn((
                                PartyMemberMpText { index: i },
                                Text::new(""),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.6, 0.8, 1.0)),
                                Node {
                                    margin: UiRect::bottom(Val::Px(2.0)),
                                    ..default()
                                },
                            ));

                            // HPバー
                            member_col
                                .spawn((
                                    Node {
                                        width: Val::Px(70.0),
                                        height: Val::Px(6.0),
                                        ..default()
                                    },
                                    BackgroundColor(hp_bar_bg),
                                ))
                                .with_children(|bar_bg| {
                                    bar_bg.spawn((
                                        PartyMemberHpBarFill { index: i },
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(hp_bar_green),
                                    ));
                                });
                        });
                }
            });

            // 右: コマンド
            area.spawn(Node {
                width: Val::Percent(40.0),
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
                    Text::new("  じゅもん"),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(unselected_color),
                ));
                cmd_area.spawn((
                    CommandCursor { index: 2 },
                    Text::new("  どうぐ"),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(unselected_color),
                ));
                cmd_area.spawn((
                    CommandCursor { index: 3 },
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
}

pub fn cleanup_battle_scene(
    mut commands: Commands,
    query: Query<Entity, With<BattleSceneRoot>>,
    mut player_query: Query<(Entity, &TilePosition, &mut Transform), With<Player>>,
    mut move_state: ResMut<MovementState>,
    game_state: Res<BattleGameState>,
    mut party_state: ResMut<PartyState>,
    active_map: Res<ActiveMap>,
) {
    // 戦闘結果のHP/MPを永続状態に書き戻す
    for (i, member) in game_state.state.party.iter().enumerate() {
        if let Some(persistent) = party_state.members.get_mut(i) {
            persistent.stats.hp = if member.stats.hp <= 0 {
                1 // 戦闘不能メンバーはHP=1で生存（全滅処理は別タスク）
            } else {
                member.stats.hp
            };
            persistent.stats.mp = member.stats.mp;
        }
    }

    // インベントリを書き戻す
    party_state.inventory = game_state.state.inventory.clone();

    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<BattleGameState>();
    commands.remove_resource::<BattleUIState>();

    // プレイヤーの移動関連コンポーネントをクリーンアップ
    if let Ok((entity, tile_pos, mut transform)) = player_query.single_mut() {
        commands
            .entity(entity)
            .remove::<MovementLocked>()
            .remove::<movement_ui::SmoothMove>()
            .remove::<PendingMove>()
            .remove::<Bounce>();

        // タイル座標に基づいて正確な位置にスナップ
        let (world_x, world_y) = active_map.to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }

    // 移動状態をリセット
    *move_state = MovementState::default();
}
