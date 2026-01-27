# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Language

日本語で対応してください。

## Build Commands

```bash
cargo build          # ビルド（rsファイル修正後は必ず実行）
cargo run            # 実行
cargo clippy         # リント
cargo test           # テスト
```

**重要**: `.rs`ファイルを修正した後は必ず`cargo build`を実行してコンパイルエラーを確認すること。

## Architecture

Bevy 0.18を使用した2Dローグライク風JRPGのプロトタイプ。

### モジュール構成

```
src/
├── main.rs              # エントリーポイント
├── game/                # ゲームシステム（ロジック層）
│   ├── map/             # マップ生成ロジック
│   │   ├── terrain.rs   # 地形タイプ定義
│   │   └── generation.rs# 生成アルゴリズム
│   └── movement/        # 移動ロジック
│       ├── events.rs    # システム間通信用メッセージ
│       └── player.rs    # タイル位置管理、移動判定
└── ui/                  # UI層（Bevy依存）
    ├── constants.rs     # 表示定数（タイルサイズ等）
    ├── camera.rs        # カメラ制御
    ├── rendering.rs     # スプライト描画
    ├── player_view.rs   # プレイヤー座標更新
    └── bounce.rs        # バウンスアニメーション
```

### 設計原則

#### 1. ゲームロジックとUIの分離

- **game/** - ゲームの「ルール」を扱う（タイル座標、移動可否判定、地形生成）
- **ui/** - 「見た目」を扱う（ワールド座標、アニメーション、描画）
- 両者の通信は **Message** で疎結合に

```rust
// game側: 移動不可を通知
blocked_events.write(MovementBlockedEvent { entity, direction });

// ui側: アニメーションで応答
fn start_bounce(mut events: MessageReader<MovementBlockedEvent>) { ... }
```

#### 2. 座標系の分離

| 層 | 座標系 | 責務 |
|---|---|---|
| game | タイル座標 (usize, usize) | 論理的な位置、当たり判定 |
| ui | ワールド座標 (f32, f32) | 画面表示、アニメーション |

#### 3. 定数の配置

- **ゲームルールの定数** → `game/`内（MAP_WIDTH, MAP_HEIGHT）
- **表示の定数** → `ui/constants.rs`（TILE_SIZE, WINDOW_SIZE）

### マップ生成システム

150x150のタイルグリッドで、以下の地形タイプを持つ：
- **Sea** - 海（デフォルト）
- **Plains** - 平地（陸地のベース）
- **Forest** - 森林（クラスター散布）
- **Mountain** - 山岳（クラスター散布）

生成アルゴリズム:
1. ランダムなシード位置から複数の島を成長させる（フロンティア拡散法）
2. 陸地タイルが目標数に達するまで拡散
3. 森林・山岳をクラスター状に散布
4. グリッドは端でラップ（トーラス状）

## 専門エージェント・スキル

- `.claude/agents/software-architect.md` - アーキテクチャ設計
- `.claude/agents/map-generation-expert.md` - マップ生成
- `.claude/skills/bevy-0.18-patterns.md` - Bevy APIパターン集
- `.claude/skills/architecture-patterns.md` - 設計パターン集
