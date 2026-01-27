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

### コード構成

- `src/main.rs` - Bevyアプリのエントリーポイント、カメラセットアップ、マップのスプライト描画
- `src/map.rs` - 手続き型マップ生成ロジック（島生成、地形クラスター配置）

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

### 専門エージェント

マップ生成に関する作業は`.claude/agents/map-generation-expert.md`で定義された専門エージェントを使用できる。
