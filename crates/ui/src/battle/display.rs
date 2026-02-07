use bevy::prelude::*;

use super::scene::{BattlePhase, BattleResource};

/// 敵HP表示テキストのマーカー
#[derive(Component)]
pub struct EnemyHpText;

/// メッセージ表示テキストのマーカー
#[derive(Component)]
pub struct MessageText;

/// プレイヤーHP表示テキストのマーカー
#[derive(Component)]
pub struct PlayerHpText;

/// コマンドカーソルのマーカー（indexでどのコマンドかを識別）
#[derive(Component)]
pub struct CommandCursor {
    pub index: usize,
}

/// 敵HPバーの前景（塗り部分）マーカー
#[derive(Component)]
pub struct EnemyHpBarFill;

/// プレイヤーHPバーの前景（塗り部分）マーカー
#[derive(Component)]
pub struct PlayerHpBarFill;

/// HP割合に応じた色を返す（>50%=緑, >25%=黄, それ以下=赤）
fn hp_bar_color(ratio: f32) -> Color {
    if ratio > 0.5 {
        Color::srgb(0.2, 0.8, 0.2)
    } else if ratio > 0.25 {
        Color::srgb(0.9, 0.8, 0.1)
    } else {
        Color::srgb(0.9, 0.2, 0.2)
    }
}

const COMMAND_COLOR_SELECTED: Color = Color::srgb(1.0, 0.9, 0.2);
const COMMAND_COLOR_UNSELECTED: Color = Color::srgb(0.6, 0.6, 0.6);

/// 戦闘画面の表示を更新するシステム
#[allow(clippy::type_complexity)]
pub fn battle_display_system(
    battle_res: Res<BattleResource>,
    mut enemy_hp_query: Query<
        &mut Text,
        (
            With<EnemyHpText>,
            Without<MessageText>,
            Without<PlayerHpText>,
            Without<CommandCursor>,
        ),
    >,
    mut message_query: Query<
        &mut Text,
        (
            With<MessageText>,
            Without<EnemyHpText>,
            Without<PlayerHpText>,
            Without<CommandCursor>,
        ),
    >,
    mut player_hp_query: Query<
        &mut Text,
        (
            With<PlayerHpText>,
            Without<EnemyHpText>,
            Without<MessageText>,
            Without<CommandCursor>,
        ),
    >,
    mut command_query: Query<
        (&CommandCursor, &mut Text, &mut TextColor),
        (
            Without<EnemyHpText>,
            Without<MessageText>,
            Without<PlayerHpText>,
        ),
    >,
    mut enemy_bar_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<EnemyHpBarFill>, Without<PlayerHpBarFill>),
    >,
    mut player_bar_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<PlayerHpBarFill>, Without<EnemyHpBarFill>),
    >,
) {
    // 敵HP更新
    let enemy = &battle_res.state.enemy;
    let enemy_ratio = enemy.stats.hp as f32 / enemy.stats.max_hp as f32;
    for mut text in &mut enemy_hp_query {
        **text = format!(
            "{} HP: {}/{}",
            enemy.kind.name(),
            enemy.stats.hp,
            enemy.stats.max_hp
        );
    }
    for (mut node, mut bg) in &mut enemy_bar_query {
        node.width = Val::Percent(enemy_ratio * 100.0);
        *bg = BackgroundColor(hp_bar_color(enemy_ratio));
    }

    // プレイヤーHP更新
    let player = &battle_res.state.player;
    let player_ratio = player.hp as f32 / player.max_hp as f32;
    for mut text in &mut player_hp_query {
        **text = format!("HP: {}/{}", player.hp, player.max_hp);
    }
    for (mut node, mut bg) in &mut player_bar_query {
        node.width = Val::Percent(player_ratio * 100.0);
        *bg = BackgroundColor(hp_bar_color(player_ratio));
    }

    // メッセージ更新
    for mut text in &mut message_query {
        match &battle_res.phase {
            BattlePhase::CommandSelect => {
                **text = "コマンド？".to_string();
            }
            BattlePhase::ShowMessage { messages, index } => {
                if let Some(msg) = messages.get(*index) {
                    **text = msg.clone();
                }
            }
            BattlePhase::BattleOver { message } => {
                **text = message.clone();
            }
        }
    }

    // コマンドカーソル更新（テキスト＋色）
    let commands = ["たたかう", "にげる"];
    for (cursor, mut text, mut color) in &mut command_query {
        let is_selected = cursor.index == battle_res.selected_command;
        let prefix = if is_selected { "> " } else { "  " };
        **text = format!("{}{}", prefix, commands[cursor.index]);
        *color = if is_selected {
            TextColor(COMMAND_COLOR_SELECTED)
        } else {
            TextColor(COMMAND_COLOR_UNSELECTED)
        };
    }
}
