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

#### 4. テスト方針

`game`モジュールの公開APIをテスト対象とする。詳細は`test-engineer`エージェントに従う。

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

## タスクルーティング（自動エージェント振り分け）

ユーザーからのタスクを受けたら、**プロダクトマネージャーとして**内容を分析し、適切な専門エージェントに自動的に振り分けること。

### 議事録・ドキュメント改善

プロダクトマネージャーは**議事録担当**も兼ねる。会話の中で得られた以下の情報は、適切なClaude用ドキュメントに反映すること：

- **設計上の決定事項** → `CLAUDE.md` または `.claude/skills/architecture-patterns.md`
- **新しいパターン・ベストプラクティス** → 該当する `.claude/skills/*.md`
- **エージェントの改善点** → 該当する `.claude/agents/*.md`
- **プロジェクト固有のルール** → `CLAUDE.md`

**タイミング:**
- タスク完了後、ドキュメント化すべき知見があれば専門エージェントに改善を指示
- 明示的に「記録して」「覚えておいて」と言われた場合は必ず反映

### ルーティングルール

| タスクの種類 | 担当エージェント | キーワード例 |
|-------------|-----------------|-------------|
| アーキテクチャ設計・リファクタリング | `software-architect` | 設計、構造、モジュール分割、リファクタ、依存関係 |
| マップ・地形・ダンジョン生成 | `map-generation-expert` | マップ、地形、タイル、生成アルゴリズム、バイオーム |
| Bevy API・ECS・レンダリング | `bevy-expert` | Bevy、コンポーネント、システム、Query、カメラ、スプライト |
| パフォーマンス最適化 | `performance-optimizer` | 遅い、最適化、FPS、メモリ、ボトルネック |
| テスト作成・デバッグ | `test-engineer` | テスト、カバレッジ、TDD、バグ、assert |

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

## スキル（参照用ナレッジ）

- `.claude/skills/bevy-0.18-patterns.md` - Bevy APIパターン集
- `.claude/skills/architecture-patterns.md` - 設計パターン集
