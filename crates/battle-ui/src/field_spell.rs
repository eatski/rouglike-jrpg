use bevy::prelude::*;

use battle::spell::{available_spells, calculate_heal_amount, SpellKind};
use shared_ui::{FieldSpellMenuOpen, PartyState};

/// フィールド呪文メニューのフェーズ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldSpellPhase {
    /// キャスター選択
    CasterSelect,
    /// 呪文選択
    SpellSelect,
    /// ターゲット選択
    TargetSelect,
    /// メッセージ表示
    ShowMessage,
}

/// フィールド呪文メニューの状態
#[derive(Resource)]
pub struct FieldSpellState {
    phase: FieldSpellPhase,
    /// キャスター候補（生存メンバーのインデックス）
    caster_candidates: Vec<usize>,
    /// キャスター選択カーソル
    caster_cursor: usize,
    /// 選択中キャスターのインデックス（party_state.members上）
    selected_caster: usize,
    /// 呪文候補
    spell_candidates: Vec<SpellKind>,
    /// 呪文選択カーソル
    spell_cursor: usize,
    /// ターゲット候補（生存メンバーのインデックス）
    target_candidates: Vec<usize>,
    /// ターゲット選択カーソル
    target_cursor: usize,
    /// 表示メッセージ
    message: String,
}

/// フィールド呪文メニューのUIルートマーカー
#[derive(Component)]
pub struct FieldSpellRoot;

/// フィールド呪文メニューのタイトルテキスト
#[derive(Component)]
pub struct FieldSpellTitle;

/// フィールド呪文メニューの選択肢アイテム
#[derive(Component)]
pub struct FieldSpellMenuItem {
    index: usize,
}

const MAX_MENU_ITEMS: usize = 4;
const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
const UNSELECTED_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const DISABLED_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);

/// メニューUIをスポーンする
fn spawn_menu_ui(commands: &mut Commands, asset_server: &AssetServer) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Bold.ttf");
    let panel_bg = Color::srgba(0.1, 0.1, 0.15, 0.92);
    let border_color = Color::srgb(0.4, 0.4, 0.5);

    commands
        .spawn((
            FieldSpellRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
            GlobalZIndex(200),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(260.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|menu_box| {
                    // タイトル
                    menu_box.spawn((
                        FieldSpellTitle,
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                    ));

                    // メニューアイテム（最大4つ）
                    for i in 0..MAX_MENU_ITEMS {
                        menu_box.spawn((
                            FieldSpellMenuItem { index: i },
                            Text::new(""),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(UNSELECTED_COLOR),
                            Visibility::Hidden,
                        ));
                    }
                });
        });
}

/// メニューUIをdespawnする
fn despawn_menu_ui(commands: &mut Commands, query: &Query<Entity, With<FieldSpellRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// フィールド呪文メニューの入力処理システム
#[allow(clippy::too_many_arguments)]
pub fn field_spell_input_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut menu_open: ResMut<FieldSpellMenuOpen>,
    mut party_state: ResMut<PartyState>,
    state: Option<ResMut<FieldSpellState>>,
    root_query: Query<Entity, With<FieldSpellRoot>>,
) {
    if !menu_open.0 {
        // メニュー非表示: 確認キーで開く
        if input_ui::is_confirm_just_pressed(&keyboard) {
            let caster_candidates: Vec<usize> = party_state
                .members
                .iter()
                .enumerate()
                .filter(|(_, m)| m.stats.is_alive())
                .map(|(i, _)| i)
                .collect();

            if caster_candidates.is_empty() {
                return;
            }

            commands.insert_resource(FieldSpellState {
                phase: FieldSpellPhase::CasterSelect,
                caster_candidates,
                caster_cursor: 0,
                selected_caster: 0,
                spell_candidates: Vec::new(),
                spell_cursor: 0,
                target_candidates: Vec::new(),
                target_cursor: 0,
                message: String::new(),
            });

            spawn_menu_ui(&mut commands, &asset_server);
            menu_open.0 = true;
        }
        return;
    }

    let Some(mut state) = state else { return };

    match state.phase.clone() {
        FieldSpellPhase::CasterSelect => {
            let count = state.caster_candidates.len();

            if input_ui::is_up_just_pressed(&keyboard) && state.caster_cursor > 0 {
                state.caster_cursor -= 1;
            }
            if input_ui::is_down_just_pressed(&keyboard) && state.caster_cursor < count - 1 {
                state.caster_cursor += 1;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                // メニューを閉じる
                despawn_menu_ui(&mut commands, &root_query);
                commands.remove_resource::<FieldSpellState>();
                menu_open.0 = false;
                return;
            }

            if input_ui::is_confirm_just_pressed(&keyboard) {
                let member_idx = state.caster_candidates[state.caster_cursor];
                state.selected_caster = member_idx;

                let spells = available_spells(party_state.members[member_idx].kind);
                if spells.is_empty() {
                    // 呪文なし → メッセージ表示
                    state.message = "じゅもんを おぼえていない".to_string();
                    state.phase = FieldSpellPhase::ShowMessage;
                } else {
                    state.spell_candidates = spells;
                    state.spell_cursor = 0;
                    state.phase = FieldSpellPhase::SpellSelect;
                }
            }
        }
        FieldSpellPhase::SpellSelect => {
            let count = state.spell_candidates.len();

            if input_ui::is_up_just_pressed(&keyboard) && state.spell_cursor > 0 {
                state.spell_cursor -= 1;
            }
            if input_ui::is_down_just_pressed(&keyboard) && state.spell_cursor < count - 1 {
                state.spell_cursor += 1;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                // キャスター選択に戻る
                state.phase = FieldSpellPhase::CasterSelect;
                return;
            }

            if input_ui::is_confirm_just_pressed(&keyboard) {
                let spell = state.spell_candidates[state.spell_cursor];
                let caster = &party_state.members[state.selected_caster];

                if caster.stats.mp < spell.mp_cost() {
                    // MP不足 → 何もしない
                    return;
                }

                if spell.is_offensive() {
                    // 攻撃呪文 → メッセージ表示
                    state.message = "フィールドでは つかえない".to_string();
                    state.phase = FieldSpellPhase::ShowMessage;
                } else {
                    // 回復呪文 → ターゲット選択
                    let target_candidates: Vec<usize> = party_state
                        .members
                        .iter()
                        .enumerate()
                        .filter(|(_, m)| m.stats.is_alive())
                        .map(|(i, _)| i)
                        .collect();
                    state.target_candidates = target_candidates;
                    state.target_cursor = 0;
                    state.phase = FieldSpellPhase::TargetSelect;
                }
            }
        }
        FieldSpellPhase::TargetSelect => {
            let count = state.target_candidates.len();

            if input_ui::is_up_just_pressed(&keyboard) && state.target_cursor > 0 {
                state.target_cursor -= 1;
            }
            if input_ui::is_down_just_pressed(&keyboard) && state.target_cursor < count - 1 {
                state.target_cursor += 1;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                // 呪文選択に戻る
                state.phase = FieldSpellPhase::SpellSelect;
                return;
            }

            if input_ui::is_confirm_just_pressed(&keyboard) {
                let spell = state.spell_candidates[state.spell_cursor];
                let target_idx = state.target_candidates[state.target_cursor];
                let caster_idx = state.selected_caster;

                // MP消費
                let consumed = party_state.members[caster_idx].stats.use_mp(spell.mp_cost());
                if !consumed {
                    return;
                }

                // 回復量計算
                let random_factor = 0.8 + rand::random::<f32>() * 0.4;
                let amount = calculate_heal_amount(spell.power(), random_factor);

                // HP回復
                let target = &mut party_state.members[target_idx];
                target.stats.hp = (target.stats.hp + amount).min(target.stats.max_hp);

                // メッセージ生成
                let caster_name = party_state.members[caster_idx].kind.name();
                let target_name = party_state.members[target_idx].kind.name();
                state.message = format!(
                    "{}は {}を となえた！\n{}の HPが {}かいふく！",
                    caster_name,
                    spell.name(),
                    target_name,
                    amount
                );
                state.phase = FieldSpellPhase::ShowMessage;
            }
        }
        FieldSpellPhase::ShowMessage => {
            if input_ui::is_confirm_just_pressed(&keyboard) {
                // キャスター選択に戻る
                state.caster_cursor = 0;
                state.phase = FieldSpellPhase::CasterSelect;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                // メニューを閉じる
                despawn_menu_ui(&mut commands, &root_query);
                commands.remove_resource::<FieldSpellState>();
                menu_open.0 = false;
            }
        }
    }
}

/// フィールド呪文メニューの表示更新システム
pub fn field_spell_display_system(
    state: Option<Res<FieldSpellState>>,
    party_state: Res<PartyState>,
    mut title_query: Query<&mut Text, (With<FieldSpellTitle>, Without<FieldSpellMenuItem>)>,
    mut item_query: Query<
        (&FieldSpellMenuItem, &mut Text, &mut TextColor, &mut Visibility),
        Without<FieldSpellTitle>,
    >,
) {
    let Some(state) = state else { return };

    // タイトル更新
    for mut text in &mut title_query {
        match state.phase {
            FieldSpellPhase::CasterSelect => {
                **text = "だれが じゅもんを つかう？".to_string();
            }
            FieldSpellPhase::SpellSelect => {
                let name = party_state.members[state.selected_caster].kind.name();
                **text = format!("{}の じゅもん", name);
            }
            FieldSpellPhase::TargetSelect => {
                **text = "だれに つかう？".to_string();
            }
            FieldSpellPhase::ShowMessage => {
                **text = state.message.clone();
            }
        }
    }

    // メニューアイテム更新
    for (item, mut text, mut color, mut vis) in &mut item_query {
        match state.phase {
            FieldSpellPhase::CasterSelect => {
                if item.index < state.caster_candidates.len() {
                    let member_idx = state.caster_candidates[item.index];
                    let member = &party_state.members[member_idx];
                    let is_selected = item.index == state.caster_cursor;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!(
                        "{}{} HP:{}/{} MP:{}/{}",
                        prefix,
                        member.kind.name(),
                        member.stats.hp,
                        member.stats.max_hp,
                        member.stats.mp,
                        member.stats.max_mp,
                    );
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            FieldSpellPhase::SpellSelect => {
                if item.index < state.spell_candidates.len() {
                    let spell = state.spell_candidates[item.index];
                    let is_selected = item.index == state.spell_cursor;
                    let caster_mp = party_state.members[state.selected_caster].stats.mp;
                    let can_use = caster_mp >= spell.mp_cost();
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!("{}{} ({})", prefix, spell.name(), spell.mp_cost());
                    *color = if !can_use {
                        TextColor(DISABLED_COLOR)
                    } else if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            FieldSpellPhase::TargetSelect => {
                if item.index < state.target_candidates.len() {
                    let member_idx = state.target_candidates[item.index];
                    let member = &party_state.members[member_idx];
                    let is_selected = item.index == state.target_cursor;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!(
                        "{}{} HP:{}/{}",
                        prefix,
                        member.kind.name(),
                        member.stats.hp,
                        member.stats.max_hp,
                    );
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            FieldSpellPhase::ShowMessage => {
                // メッセージ表示中はアイテム非表示
                *vis = Visibility::Hidden;
            }
        }
    }
}
