# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Language

日本語で対応してください。

## Build Commands

```bash
cargo build                       # ビルド（rsファイル修正後は必ず実行）
cargo run                         # 実行
cargo run -- --screenshot          # スクショ自動撮影（screenshots/latest.png）
cargo run -p generate-tiles        # アセット生成（タイルスプライト）
cargo clippy --workspace          # リント（全crate）
cargo test --workspace            # テスト（全crate）
```

**重要**: `.rs`ファイルを修正した後は必ず`cargo build`を実行してコンパイルエラーを確認すること。

## Architecture

Bevy 0.18を使用した2Dローグライク風JRPGのプロトタイプ。Cargoワークスペース構成。

- **game crate**: 純粋Rust（Bevy非依存）、ゲームロジック
- **ui crate**: Bevy依存、描画・入力・アニメーション
- **generate_tiles crate**: アセット生成ツール（独立バイナリ、image/rand依存）

詳細は `software-architect` エージェントを参照。

## PMフロー

```
ユーザー指示 → PM → エージェント → PM → ユーザーに報告
```

- **コミット**: ユーザーが明示的に指示した場合のみ `task-committer` を起動
- **Explore**: 専門エージェントが内部で使用（PMは直接起動しない）
- **複合タスク**: まず `software-architect` で設計、その後各エージェントに委譲
- **ビジュアル確認**: 画面に関わる変更は各エージェントが `cargo run -- --screenshot` → Read で自主的に確認する。必要に応じて `screenshot-reviewer` を起動

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
| `screenshot-reviewer` | スクショ撮影＆ビジュアルFB |
| `task-committer` | コミット・ドキュメント整理 |
