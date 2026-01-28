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
            └── bounce.rs         # バウンスアニメーション
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

#### 1. ゲームロジックとUIの完全分離（crate境界）

- **game crate** - 純粋Rust。Bevy非依存。ゲームの「ルール」のみ
  - タイル座標、移動可否判定、地形生成
  - 他のゲームエンジンでも再利用可能
- **ui crate** - Bevy依存。「見た目」と「Bevy統合」
  - ワールド座標、アニメーション、描画
  - Component, Resource, Message等のBevy型
- 両者の通信は **ui側のMessage** で疎結合に

```rust
// game側: 純粋関数で判定のみ
match game::movement::try_move(x, y, dx, dy, &grid) {
    MoveResult::Blocked => { /* ... */ }
    MoveResult::Moved { new_x, new_y } => { /* ... */ }
}

// ui側: Bevyのイベントシステムで通知
blocked_events.write(MovementBlockedEvent { entity, direction });
```

#### 2. 座標系の分離

| crate | 座標系 | 責務 |
|---|---|---|
| game | タイル座標 (usize, usize) | 論理的な位置、当たり判定 |
| ui | ワールド座標 (f32, f32) | 画面表示、アニメーション |

#### 3. 定数の配置

- **ゲームルールの定数** → `crates/game/`内（MAP_WIDTH, MAP_HEIGHT）
- **表示の定数** → `crates/ui/src/constants.rs`（TILE_SIZE, WINDOW_SIZE）

#### 4. テスト方針

`game` crateの公開APIをテスト対象とする。Bevy非依存のため単体テストが容易。詳細は`test-engineer`エージェントに従う。

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

### アセット生成システム

ゲームアセット（ドット絵スプライト）は`cargo run --bin generate-tiles`で生成する。

#### 生成されるアセット

- **assets/tiles/sea.png** - 16x16 海タイル（深い青、波模様）
- **assets/tiles/plains.png** - 16x16 平地タイル（草緑、草テクスチャ）
- **assets/tiles/forest.png** - 16x16 森林タイル（濃い緑、木のシルエット）
- **assets/tiles/mountain.png** - 16x16 山岳タイル（灰色、岩と雪）

#### 確認方法

```bash
cargo run --bin generate-tiles  # アセット生成
# AIエージェントはReadツールで直接画像を確認可能
```

#### Bevyでの読み込み

```rust
// ui/src/resources.rs
#[derive(Resource)]
pub struct TileTextures {
    pub sea: Handle<Image>,
    pub plains: Handle<Image>,
    pub forest: Handle<Image>,
    pub mountain: Handle<Image>,
}

// ui/src/rendering.rs
fn load_tile_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TileTextures {
        sea: asset_server.load("tiles/sea.png"),
        plains: asset_server.load("tiles/plains.png"),
        forest: asset_server.load("tiles/forest.png"),
        mountain: asset_server.load("tiles/mountain.png"),
    });
}
```

## タスクルーティング（自動エージェント振り分け）

ユーザーからのタスクを受けたら、**プロダクトマネージャーとして**内容を分析し、適切な専門エージェントに自動的に振り分けること。

### 議事録・ドキュメント改善

タスク完了後のドキュメント更新は `documentation-keeper` エージェントに委譲する。

#### フロー

```
1. 専門エージェントがタスク完了
   ↓
2. PMが documentation-keeper を起動
   ↓
3. ドキュメント更新の要否を判断・実行
```

### ルーティングルール

| タスクの種類 | 担当エージェント | キーワード例 |
|-------------|-----------------|-------------|
| アーキテクチャ設計・リファクタリング | `software-architect` | 設計、構造、モジュール分割、リファクタ、依存関係 |
| マップ・地形・ダンジョン生成 | `map-generation-expert` | マップ、地形、タイル、生成アルゴリズム、バイオーム |
| Bevy API・ECS・レンダリング | `bevy-expert` | Bevy、コンポーネント、システム、Query、カメラ、スプライト |
| パフォーマンス最適化 | `performance-optimizer` | 遅い、最適化、FPS、メモリ、ボトルネック |
| テスト作成・デバッグ | `test-engineer` | テスト、カバレッジ、TDD、バグ、assert |
| ピクセルアート・アセット生成 | `pixel-art-generator` | ドット絵、スプライト、タイル画像、アセット生成 |
| ドキュメント更新（タスク完了後） | `documentation-keeper` | ※PMが自動で起動 |

### 振り分けフロー

```
1. タスク受信
   ↓
2. 内容を分析し、最適な専門エージェントを判定
   ↓
3. Task toolで専門エージェントを起動
   ↓
4. 結果をユーザーに報告
```

### Exploreエージェントの位置づけ

**重要**: `Explore`は専門エージェントではなく、**手段**として位置づける。

```
❌ 誤: ユーザー → PM → Explore（直接起動）
⭕ 正: ユーザー → PM → 専門エージェント → Explore（内部で使用）
```

- PMがExploreを直接起動しない
- 専門エージェント（test-engineer, software-architect等）がタスク遂行中に必要に応じてExploreを呼び出す
- 調査・洗い出し系のタスクも、まず該当する専門エージェントに振り分ける

### 複合タスクの場合

複数の専門領域にまたがるタスクは：
1. まず `software-architect` で全体設計を固める
2. 各専門エージェントに実装を委譲
3. 最後に統合・レビュー

### エージェントが存在しない領域

新しい専門領域のタスクが発生した場合：
1. 必要に応じて新規エージェントを `.claude/agents/` に作成
2. CLAUDE.mdのルーティングルールに追加
3. タスクを実行

## 専門エージェント一覧

| エージェント | ファイル | 専門領域 |
|-------------|---------|---------|
| Software Architect | `.claude/agents/software-architect.md` | アーキテクチャ設計、リファクタリング |
| Map Generation Expert | `.claude/agents/map-generation-expert.md` | 手続き型マップ生成 |
| Bevy Expert | `.claude/agents/bevy-expert.md` | Bevy 0.18 API、ECS |
| Performance Optimizer | `.claude/agents/performance-optimizer.md` | パフォーマンス分析・改善 |
| Test Engineer | `.claude/agents/test-engineer.md` | テスト戦略・実装 |
| Pixel Art Generator | `.claude/agents/pixel-art-generator.md` | ピクセルアート・アセット生成 |
| Documentation Keeper | `.claude/agents/documentation-keeper.md` | 議事録・ドキュメント管理 |

## スキル（参照用ナレッジ）

- `.claude/skills/bevy-0.18-patterns.md` - Bevy APIパターン集
- `.claude/skills/architecture-patterns.md` - 設計パターン集
