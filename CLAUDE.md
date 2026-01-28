# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Language

日本語で対応してください。

## Build Commands

```bash
cargo build                       # ビルド（rsファイル修正後は必ず実行）
cargo run                         # 実行
cargo run --bin generate-tiles    # アセット生成（タイルスプライト）
cargo clippy --workspace          # リント（全crate）
cargo test --workspace            # テスト（全crate）
```

**重要**: `.rs`ファイルを修正した後は必ず`cargo build`を実行してコンパイルエラーを確認すること。

## Architecture

Bevy 0.18を使用した2Dローグライク風JRPGのプロトタイプ。

### プロジェクト構成（Cargoワークスペース）

```
rouglike-jrpg/
├── Cargo.toml               # ワークスペース定義、依存: bevy, game, ui, image, rand
├── src/
│   ├── main.rs              # エントリーポイント
│   └── bin/
│       └── generate_tiles.rs # アセット生成バイナリ（image, rand使用）
├── assets/                  # ゲームアセット
│   └── tiles/               # 地形タイルスプライト（16x16 PNG）
│       ├── sea.png
│       ├── plains.png
│       ├── forest.png
│       └── mountain.png
└── crates/
    ├── game/                # ゲームロジック層【Bevy非依存・純粋Rust】
    │   ├── Cargo.toml       # 依存: rand のみ
    │   └── src/
    │       ├── lib.rs
    │       ├── map/
    │       │   ├── terrain.rs    # 地形タイプ定義
    │       │   └── generation.rs # 生成アルゴリズム、MapData
    │       └── movement/
    │           ├── events.rs     # Direction型
    │           └── player.rs     # try_move(), is_passable(), MoveResult
    └── ui/                  # UI層【Bevy依存】
        ├── Cargo.toml       # 依存: bevy, game
        └── src/
            ├── lib.rs
            ├── components.rs     # Player, TilePosition, MovementLocked
            ├── resources.rs      # MapDataResource, MovementState, TileTextures
            ├── events.rs         # MovementBlockedEvent, PlayerMovedEvent
            ├── player_input.rs   # 入力処理システム
            ├── constants.rs      # 表示定数（TILE_SIZE等）
            ├── camera.rs         # カメラ制御
            ├── rendering.rs      # スプライト描画（テクスチャベース）
            ├── player_view.rs    # プレイヤー座標更新
            ├── bounce.rs         # バウンスアニメーション（移動不可時）
            └── smooth_move.rs    # 滑らか移動アニメーション（移動成功時）
```

### 依存関係

```
rouglike-jrpg (binary)
    ├── bevy 0.18
    ├── image 0.24 (アセット生成用)
    ├── rand 0.8 (アセット生成用)
    ├── game (純粋Rust - 他エンジンでも再利用可能)
    └── ui → game

generate-tiles (binary)
    ├── image 0.24
    └── rand 0.8
```

### 設計原則

- **game crate**: 純粋Rust（Bevy非依存）、ゲームロジックのみ
- **ui crate**: Bevy依存、描画・アニメーション

詳細は `software-architect` エージェントを参照。

### マップ生成

詳細は `map-generation-expert` エージェントを参照。

### アセット生成

詳細は `pixel-art-generator` エージェントを参照。

## PMフロー

```
ユーザー指示 → PM → エージェント → PM → ユーザーに報告
```

- **コミット**: ユーザーが明示的に指示した場合のみ `commit-keeper` を起動
- **Explore**: 専門エージェントが内部で使用（PMは直接起動しない）
- **複合タスク**: まず `software-architect` で設計、その後各エージェントに委譲

## 専門エージェント

| エージェント | 担当領域 |
|-------------|---------|
| `software-architect` | 設計、リファクタリング |
| `map-generation-expert` | マップ・地形生成 |
| `bevy-expert` | Bevy API、ECS |
| `performance-optimizer` | パフォーマンス最適化 |
| `test-engineer` | テスト作成・デバッグ |
| `pixel-art-generator` | ドット絵・アセット生成 |
| `commit-keeper` | ドキュメント更新・コミット |

## スキル

- `.claude/skills/bevy-0.18-patterns.md`
- `.claude/skills/architecture-patterns.md`
