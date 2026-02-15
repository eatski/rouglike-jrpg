# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Language

日本語で対応してください。

## Build Commands

```bash
cargo build                       # ビルド（rsファイル修正後は必ず実行）
cargo run                         # 実行
cargo run -p generate-tiles       # アセット生成（タイルスプライト）
cargo clippy --workspace          # リント（全crate）
cargo test --workspace            # テスト（全crate）
```

**重要**: `.rs`ファイルを修正した後は必ず`cargo build`を実行してコンパイルエラーを確認すること。

## Architecture

Bevy 0.18を使用した2Dローグライク風JRPGのプロトタイプ。Cargoワークスペース構成（17 crate）。

**ドメイン層（依存なし）**:
- **terrain**: 地形・座標・方向（Terrain, Position, Direction）
- **party**: パーティ・キャラクター・ステータス・アイテム・仲間募集（PartyMember（各キャラ個別インベントリ）、CombatStats、ItemKind、Inventory、INVENTORY_CAPACITY=6、RecruitCandidate、RecruitmentStatus: Undiscovered→Acquaintance→Recruited）
- **battle**: 戦闘ロジック（敵、魔法（クラス別呪文制限含む）、アイテム（個人所持）、戦闘処理）
- **cave**: 洞窟生成ロジック
- **town**: 街ロジック（やどや、道具屋（キャラ選択購入、容量チェック）、洞窟ヒント、仲間候補との会話）
- **world**: ワールドマップ生成・島配置・仲間候補配置
- **time**: 時間カウント（TimeCount構造体）

**UI共通層（Bevy依存）**:
- **app-state**: AppState（Exploring/Battle/Cave/Town）
- **input-ui**: 入力ソース抽象化（InputSource）
- **shared-ui**: UI共通定数・リソース（PartyState: パーティ、所持金、仲間候補リスト）
- **movement-ui**: 移動メカニクス（コンポーネント、イベント、アニメーション）

**UI機能層（Bevy依存）**:
- **world-ui**: ワールドマップシーン・入力・描画・エンカウントシステム・仲間候補マーカー表示
- **cave-ui**: 洞窟シーン・入力（ワールドマップ座標からシード生成し決定的な洞窟生成、ChaCha8Rng使用）
- **town-ui**: 街シーン・入力・メニュー（やどや、道具屋（キャラ選択パネル）、ヒント、話を聞く（仲間候補イベント）、出る）
- **battle-ui**: 戦闘シーン・入力（クラス別呪文選択制限、個人インベントリ使用）・表示（呪文リスト、無効コマンド灰色表示、アイテムなし時「どうぐ」灰色）
- **time-ui**: 時間カウンター表示（右上UI、TileEnteredEventで+1、MapMode時非表示）

**ツール**:
- **generate_tiles**: タイルスプライト生成（独立バイナリ）

詳細は `software-architect` エージェントを参照。

## PMフロー

```
ユーザー指示 → PM → エージェント → PM → ユーザーに報告
```

- **コミット**: ユーザーが明示的に指示した場合のみ `task-committer` を起動
- **Explore**: 専門エージェントが内部で使用（PMは直接起動しない）
- **複合タスク**: まず `software-architect` で設計、その後各エージェントに委譲
### スキル

`.claude/skills/` にスキル定義を配置。ユーザーが直接呼び出せる便利コマンド。

- **commit**: `task-committer` を起動してコミット実行（ドキュメント整理も含む）

## 専門エージェント

| エージェント | 担当領域 |
|-------------|---------|
| `software-architect` | 設計、リファクタリング |
| `map-generation-expert` | マップ・地形生成 |
| `bevy-expert` | Bevy API、ECS |
| `performance-optimizer` | パフォーマンス最適化 |
| `test-engineer` | テスト作成・デバッグ |
| `pixel-art-generator` | ドット絵・アセット生成 |
| `qa-verifier` | 品質保証（テスト実行＋手動確認の促し） |
| `task-committer` | コミット・ドキュメント整理 |
