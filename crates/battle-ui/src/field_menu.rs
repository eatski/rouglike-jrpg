use bevy::prelude::*;

use battle::spell::{available_spells, calculate_heal_amount, SpellKind};
use app_state::{FieldMenuOpen, PartyState};
use party::{ItemEffect, ItemKind};

/// フィールドメニューのフェーズ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldMenuPhase {
    /// トップメニュー（じゅもん/どうぐ）
    TopMenu,
    /// キャスター選択（呪文フロー）
    CasterSelect,
    /// 呪文選択
    SpellSelect,
    /// メンバー選択（アイテムフロー）
    MemberSelect,
    /// アイテム選択
    ItemSelect,
    /// ターゲット選択（共用）
    TargetSelect,
    /// メッセージ表示
    ShowMessage,
}

/// フィールドメニューの状態
#[derive(Resource)]
pub struct FieldMenuState {
    phase: FieldMenuPhase,
    // トップメニュー
    top_cursor: usize,
    // 呪文フロー
    caster_candidates: Vec<usize>,
    caster_cursor: usize,
    selected_caster: usize,
    spell_candidates: Vec<SpellKind>,
    spell_cursor: usize,
    // アイテムフロー
    member_candidates: Vec<usize>,
    member_cursor: usize,
    selected_member: usize,
    item_candidates: Vec<ItemKind>,
    item_cursor: usize,
    // 共用
    target_candidates: Vec<usize>,
    target_cursor: usize,
    is_item_mode: bool,
    message: String,
}

/// フィールドメニューのUIルートマーカー
#[derive(Component)]
pub struct FieldMenuRoot;

/// フィールドメニューのタイトルテキスト
#[derive(Component)]
pub struct FieldMenuTitle;

/// フィールドメニューの選択肢アイテム
#[derive(Component)]
pub struct FieldMenuItem {
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
            FieldMenuRoot,
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
                        FieldMenuTitle,
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
                            FieldMenuItem { index: i },
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
fn despawn_menu_ui(commands: &mut Commands, query: &Query<Entity, With<FieldMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 生存メンバーのインデックス一覧を返す
fn alive_member_indices(party_state: &PartyState) -> Vec<usize> {
    party_state
        .members
        .iter()
        .enumerate()
        .filter(|(_, m)| m.stats.is_alive())
        .map(|(i, _)| i)
        .collect()
}

/// フィールドメニューの入力処理システム
#[allow(clippy::too_many_arguments)]
pub fn field_menu_input_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut menu_open: ResMut<FieldMenuOpen>,
    mut party_state: ResMut<PartyState>,
    state: Option<ResMut<FieldMenuState>>,
    root_query: Query<Entity, With<FieldMenuRoot>>,
) {
    if !menu_open.0 {
        // メニュー非表示: 確認キーで開く
        if input_ui::is_confirm_just_pressed(&keyboard) {
            let alive = alive_member_indices(&party_state);
            if alive.is_empty() {
                return;
            }

            commands.insert_resource(FieldMenuState {
                phase: FieldMenuPhase::TopMenu,
                top_cursor: 0,
                caster_candidates: Vec::new(),
                caster_cursor: 0,
                selected_caster: 0,
                spell_candidates: Vec::new(),
                spell_cursor: 0,
                member_candidates: Vec::new(),
                member_cursor: 0,
                selected_member: 0,
                item_candidates: Vec::new(),
                item_cursor: 0,
                target_candidates: Vec::new(),
                target_cursor: 0,
                is_item_mode: false,
                message: String::new(),
            });

            spawn_menu_ui(&mut commands, &asset_server);
            menu_open.0 = true;
        }
        return;
    }

    let Some(mut state) = state else { return };

    match state.phase.clone() {
        FieldMenuPhase::TopMenu => {
            if input_ui::is_up_just_pressed(&keyboard) && state.top_cursor > 0 {
                state.top_cursor -= 1;
            }
            if input_ui::is_down_just_pressed(&keyboard) && state.top_cursor < 1 {
                state.top_cursor += 1;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                despawn_menu_ui(&mut commands, &root_query);
                commands.remove_resource::<FieldMenuState>();
                menu_open.0 = false;
                return;
            }

            if input_ui::is_confirm_just_pressed(&keyboard) {
                if state.top_cursor == 0 {
                    // じゅもん → CasterSelect
                    let caster_candidates = alive_member_indices(&party_state);
                    state.caster_candidates = caster_candidates;
                    state.caster_cursor = 0;
                    state.is_item_mode = false;
                    state.phase = FieldMenuPhase::CasterSelect;
                } else {
                    // どうぐ → MemberSelect
                    let member_candidates = alive_member_indices(&party_state);
                    state.member_candidates = member_candidates;
                    state.member_cursor = 0;
                    state.is_item_mode = true;
                    state.phase = FieldMenuPhase::MemberSelect;
                }
            }
        }
        FieldMenuPhase::CasterSelect => {
            let count = state.caster_candidates.len();

            if input_ui::is_up_just_pressed(&keyboard) && state.caster_cursor > 0 {
                state.caster_cursor -= 1;
            }
            if input_ui::is_down_just_pressed(&keyboard) && state.caster_cursor < count - 1 {
                state.caster_cursor += 1;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                state.phase = FieldMenuPhase::TopMenu;
                return;
            }

            if input_ui::is_confirm_just_pressed(&keyboard) {
                let member_idx = state.caster_candidates[state.caster_cursor];
                state.selected_caster = member_idx;

                let spells = available_spells(party_state.members[member_idx].kind, party_state.members[member_idx].level);
                if spells.is_empty() {
                    state.message = "じゅもんを おぼえていない".to_string();
                    state.phase = FieldMenuPhase::ShowMessage;
                } else {
                    state.spell_candidates = spells;
                    state.spell_cursor = 0;
                    state.phase = FieldMenuPhase::SpellSelect;
                }
            }
        }
        FieldMenuPhase::SpellSelect => {
            let count = state.spell_candidates.len();

            if input_ui::is_up_just_pressed(&keyboard) && state.spell_cursor > 0 {
                state.spell_cursor -= 1;
            }
            if input_ui::is_down_just_pressed(&keyboard) && state.spell_cursor < count - 1 {
                state.spell_cursor += 1;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                state.phase = FieldMenuPhase::CasterSelect;
                return;
            }

            if input_ui::is_confirm_just_pressed(&keyboard) {
                let spell = state.spell_candidates[state.spell_cursor];
                let caster = &party_state.members[state.selected_caster];

                if caster.stats.mp < spell.mp_cost() {
                    return;
                }

                if spell.is_offensive() {
                    state.message = "フィールドでは つかえない".to_string();
                    state.phase = FieldMenuPhase::ShowMessage;
                } else {
                    let target_candidates = alive_member_indices(&party_state);
                    state.target_candidates = target_candidates;
                    state.target_cursor = 0;
                    state.phase = FieldMenuPhase::TargetSelect;
                }
            }
        }
        FieldMenuPhase::MemberSelect => {
            let count = state.member_candidates.len();

            if input_ui::is_up_just_pressed(&keyboard) && state.member_cursor > 0 {
                state.member_cursor -= 1;
            }
            if input_ui::is_down_just_pressed(&keyboard) && state.member_cursor < count - 1 {
                state.member_cursor += 1;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                state.phase = FieldMenuPhase::TopMenu;
                return;
            }

            if input_ui::is_confirm_just_pressed(&keyboard) {
                let member_idx = state.member_candidates[state.member_cursor];
                state.selected_member = member_idx;

                let items = party_state.members[member_idx].inventory.owned_items();
                if items.is_empty() {
                    state.message = "もちものが ない".to_string();
                    state.phase = FieldMenuPhase::ShowMessage;
                } else {
                    state.item_candidates = items;
                    state.item_cursor = 0;
                    state.phase = FieldMenuPhase::ItemSelect;
                }
            }
        }
        FieldMenuPhase::ItemSelect => {
            let count = state.item_candidates.len();

            if input_ui::is_up_just_pressed(&keyboard) && state.item_cursor > 0 {
                state.item_cursor -= 1;
            }
            if input_ui::is_down_just_pressed(&keyboard) && state.item_cursor < count - 1 {
                state.item_cursor += 1;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                state.phase = FieldMenuPhase::MemberSelect;
                return;
            }

            if input_ui::is_confirm_just_pressed(&keyboard) {
                let item = state.item_candidates[state.item_cursor];
                match item.effect() {
                    ItemEffect::Heal { .. } => {
                        // 回復アイテム → ターゲット選択
                        let target_candidates = alive_member_indices(&party_state);
                        state.target_candidates = target_candidates;
                        state.target_cursor = 0;
                        state.phase = FieldMenuPhase::TargetSelect;
                    }
                    ItemEffect::KeyItem => {
                        // キーアイテム → 説明表示（消費しない）
                        let member_name = party_state.members[state.selected_member].kind.name();
                        state.message = format!(
                            "{}は {}を しらべた。\n{}",
                            member_name,
                            item.name(),
                            item.description()
                        );
                        state.phase = FieldMenuPhase::ShowMessage;
                    }
                }
            }
        }
        FieldMenuPhase::TargetSelect => {
            let count = state.target_candidates.len();

            if input_ui::is_up_just_pressed(&keyboard) && state.target_cursor > 0 {
                state.target_cursor -= 1;
            }
            if input_ui::is_down_just_pressed(&keyboard) && state.target_cursor < count - 1 {
                state.target_cursor += 1;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                if state.is_item_mode {
                    state.phase = FieldMenuPhase::ItemSelect;
                } else {
                    state.phase = FieldMenuPhase::SpellSelect;
                }
                return;
            }

            if input_ui::is_confirm_just_pressed(&keyboard) {
                let target_idx = state.target_candidates[state.target_cursor];

                if state.is_item_mode {
                    // アイテム使用
                    let item = state.item_candidates[state.item_cursor];
                    let member_idx = state.selected_member;

                    if let ItemEffect::Heal { power } = item.effect() {
                        let used = party_state.members[member_idx].inventory.use_item(item);
                        if !used {
                            return;
                        }

                        let random_factor = 0.8 + rand::random::<f32>() * 0.4;
                        let amount = calculate_heal_amount(power, random_factor);

                        let target = &mut party_state.members[target_idx];
                        target.stats.hp = (target.stats.hp + amount).min(target.stats.max_hp);

                        let member_name = party_state.members[member_idx].kind.name();
                        let target_name = party_state.members[target_idx].kind.name();
                        state.message = format!(
                            "{}は {}を つかった！\n{}の HPが {}かいふく！",
                            member_name,
                            item.name(),
                            target_name,
                            amount
                        );
                    }
                } else {
                    // 呪文使用
                    let spell = state.spell_candidates[state.spell_cursor];
                    let caster_idx = state.selected_caster;

                    let consumed = party_state.members[caster_idx].stats.use_mp(spell.mp_cost());
                    if !consumed {
                        return;
                    }

                    let random_factor = 0.8 + rand::random::<f32>() * 0.4;
                    let amount = calculate_heal_amount(spell.power(), random_factor);

                    let target = &mut party_state.members[target_idx];
                    target.stats.hp = (target.stats.hp + amount).min(target.stats.max_hp);

                    let caster_name = party_state.members[caster_idx].kind.name();
                    let target_name = party_state.members[target_idx].kind.name();
                    state.message = format!(
                        "{}は {}を となえた！\n{}の HPが {}かいふく！",
                        caster_name,
                        spell.name(),
                        target_name,
                        amount
                    );
                }
                state.phase = FieldMenuPhase::ShowMessage;
            }
        }
        FieldMenuPhase::ShowMessage => {
            if input_ui::is_confirm_just_pressed(&keyboard) {
                // トップメニューに戻る
                state.top_cursor = 0;
                state.phase = FieldMenuPhase::TopMenu;
            }

            if input_ui::is_cancel_just_pressed(&keyboard) {
                despawn_menu_ui(&mut commands, &root_query);
                commands.remove_resource::<FieldMenuState>();
                menu_open.0 = false;
            }
        }
    }
}

/// フィールドメニューの表示更新システム
pub fn field_menu_display_system(
    state: Option<Res<FieldMenuState>>,
    party_state: Res<PartyState>,
    mut title_query: Query<&mut Text, (With<FieldMenuTitle>, Without<FieldMenuItem>)>,
    mut item_query: Query<
        (&FieldMenuItem, &mut Text, &mut TextColor, &mut Visibility),
        Without<FieldMenuTitle>,
    >,
) {
    let Some(state) = state else { return };

    // タイトル更新
    for mut text in &mut title_query {
        match state.phase {
            FieldMenuPhase::TopMenu => {
                **text = String::new();
            }
            FieldMenuPhase::CasterSelect => {
                **text = "だれが じゅもんを つかう？".to_string();
            }
            FieldMenuPhase::SpellSelect => {
                let name = party_state.members[state.selected_caster].kind.name();
                **text = format!("{}の じゅもん", name);
            }
            FieldMenuPhase::MemberSelect => {
                **text = "だれの どうぐを つかう？".to_string();
            }
            FieldMenuPhase::ItemSelect => {
                let name = party_state.members[state.selected_member].kind.name();
                **text = format!("{}の もちもの", name);
            }
            FieldMenuPhase::TargetSelect => {
                **text = "だれに つかう？".to_string();
            }
            FieldMenuPhase::ShowMessage => {
                **text = state.message.clone();
            }
        }
    }

    // メニューアイテム更新
    for (item, mut text, mut color, mut vis) in &mut item_query {
        match state.phase {
            FieldMenuPhase::TopMenu => {
                let labels = ["じゅもん", "どうぐ"];
                if item.index < labels.len() {
                    let is_selected = item.index == state.top_cursor;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!("{}{}", prefix, labels[item.index]);
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
            FieldMenuPhase::CasterSelect => {
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
            FieldMenuPhase::SpellSelect => {
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
            FieldMenuPhase::MemberSelect => {
                if item.index < state.member_candidates.len() {
                    let member_idx = state.member_candidates[item.index];
                    let member = &party_state.members[member_idx];
                    let is_selected = item.index == state.member_cursor;
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
            FieldMenuPhase::ItemSelect => {
                if item.index < state.item_candidates.len() {
                    let item_kind = state.item_candidates[item.index];
                    let count = party_state.members[state.selected_member]
                        .inventory
                        .count(item_kind);
                    let is_selected = item.index == state.item_cursor;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!("{}{} x{}", prefix, item_kind.name(), count);
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
            FieldMenuPhase::TargetSelect => {
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
            FieldMenuPhase::ShowMessage => {
                *vis = Visibility::Hidden;
            }
        }
    }
}
