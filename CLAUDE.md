# CLAUDE.md

日本語で対応してください。

## Tool
各ツール（ReadやEditなど）の対象を指定するパスは相対パスで指定すること

## Build Commands

```bash
cargo build                       # ビルド（rsファイル修正後は必ず実行）
cargo run                         # 実行
cargo run -p generate-tiles       # アセット生成
cargo run -p screenshot-{battle,town,field,cave,world}  # スクリーンショット撮影
cargo clippy --workspace          # リント
cargo test --workspace            # テスト
```

**重要**: `.rs`ファイルを修正した後は必ず`cargo build`を実行してコンパイルエラーを確認すること。

## Architecture

Bevy 0.18の2Dローグライク風JRPG。Cargoワークスペース構成。

ドメイン層（Bevy非依存）→ UI共通層 → UI機能層 の3層構成＋ツール群。

## ワークフロー

- **コミット**: ユーザーが明示的に指示した場合のみ `/commit` で実行
- **複合タスク**: まず `software-architect` で設計、その後各エージェントに委譲
