# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Language

日本語で対応してください。

## Build Commands

```bash
cargo build                       # ビルド（rsファイル修正後は必ず実行）
cargo run                         # 実行
cargo run -p generate-tiles       # アセット生成（タイルスプライト）
cargo run -p screenshot-battle    # 戦闘画面スクリーンショット撮影
cargo run -p screenshot-town      # 街画面スクリーンショット撮影
cargo run -p screenshot-field     # フィールド画面スクリーンショット撮影
cargo run -p screenshot-cave      # 洞窟画面スクリーンショット撮影
cargo run -p screenshot-world     # ワールドマップスクリーンショット撮影
cargo clippy --workspace          # リント（全crate）
cargo test --workspace            # テスト（全crate）
```

**重要**: `.rs`ファイルを修正した後は必ず`cargo build`を実行してコンパイルエラーを確認すること。

## Architecture

Bevy 0.18を使用した2Dローグライク風JRPGのプロトタイプ。Cargoワークスペース構成（20 crate）。

**ドメイン層（依存なし）**:
- **terrain**: 地形・座標・方向（Terrain, Position, Direction）。Mountain は歩行不可（`is_walkable()` = false）
- **party**: パーティ・キャラクター・ステータス・アイテム・装備・仲間募集・レベルシステム（PartyMember（level/expフィールド、gain_exp()でレベルアップ処理）、CombatStats（apply_growth()でステータス成長）、StatGrowth（クラス別成長値）、ItemKind（Herb/CopperKey）、ItemEffect（Heal{power}/KeyItem: effect()/description()/is_consumable()でアイテム効果分類）、Inventory、INVENTORY_CAPACITY=6、WeaponKind、Equipment、effective_attack()、RecruitCandidate、RecruitmentStatus: Undiscovered→Acquaintance→Recruited）
- **battle**: 戦闘ロジック（敵（exp_reward()で経験値報酬）、魔法（クラス別・レベル別呪文習得：SpellKind（Fire/Blaze/Heal/FullHeal）、spell_learn_table()でクラス別習得テーブル管理、available_spells(kind, level)でレベルに応じた呪文解放、spells_learned_at_level()で特定レベルの習得呪文判定）、アイテム（個人所持）、戦闘処理（effective_attack使用）、BattleState::total_exp_reward()で勝利時合計経験値計算）
- **cave**: 洞窟生成ロジック（TreasureContent/TreasureChestで宝箱定義、床タイルランダム配置（最大3個）、宝箱の中身はCopperKey固定）
- **town**: 街ロジック（やどや、よろず屋（アイテム・武器販売、キャラ選択購入、容量チェック）、洞窟ヒント、仲間候補との会話）
- **world**: ワールドマップ生成（5大陸方式：大陸成長・海岸侵食・内陸湖・極小島除去・歩行連結性保証）・島配置・仲間候補配置（大大陸3500×2、小大陸2000×3、大陸間境界バッファCONTINENT_BORDER_GAP=4.0で分離、島サイズ別街数：大島5個、小島3個、極小島100タイル未満は配置なし、街間最小間隔MIN_TOWN_DISTANCE=10タイル）
- **time**: 時間カウント（TimeCount構造体）

**UI共通層（Bevy依存）**:
- **app-state**: AppState（Exploring/Battle/Cave/Town）、SceneState（Exploring/Town/Cave/Hokora）、PartyState（パーティ、所持金、仲間候補リスト）、RecruitmentMap、FieldMenuOpen、HokoraPositions（祠のワールドマップ座標リスト）、OpenedChests（取得済み宝箱の永続管理：洞窟座標→インデックス集合のHashMap）
- **input-ui**: 入力ソース抽象化（InputSource）、確認キー消費関数（clear_confirm_just_pressed）
- **movement-ui**: 移動メカニクス（コンポーネント、イベント、アニメーション、UI共通定数、MovementState、ActiveMap、WorldMapData）

**UI機能層（Bevy依存）**:
- **world-ui**: ワールドマップシーン・入力・描画・エンカウントシステム・仲間候補マーカー表示・フィールドHUDにレベル表示（HudNameTextコンポーネント）・海岸線オートタイル（coast_lookup: 8隣接ビットマスク→47タイルインデックス変換）
- **cave-ui**: 洞窟シーン・入力（ワールドマップ座標からシード生成し決定的な洞窟生成、ChaCha8Rng使用）・宝箱表示（ChestEntity/CaveTreasures）・宝箱取得システム（check_chest_system: TileEnteredEvent検知→アイテム/武器追加→OpenedChests記録）・メッセージ表示（CaveMessageState/CaveMessageUI、cave_message_input_system/cave_message_display_system）
- **town-ui**: 街シーン・入力・メニュー（やどや、よろず屋（ShopGoods統合、キャラ選択パネル、Display::Noneパネル制御）、ヒント、話を聞く（仲間候補イベント）、出る）
- **battle-ui**: 戦闘シーン・入力（クラス別・レベル別呪文選択制限、個人インベントリ使用、KeyItemは戦闘中使用不可）・表示（呪文リスト、無効コマンド灰色表示、アイテムなし時「どうぐ」灰色、KeyItemは灰色表示）・フィールドメニュー（確認キー→トップメニュー（じゅもん/どうぐ）→キャスター/メンバー選択→呪文/アイテム選択→ターゲット選択→結果表示、KeyItemは説明表示のみ）・勝利時経験値獲得メッセージ・レベルアップメッセージ表示・レベルアップ時呪文習得メッセージ（「○○は △△を おぼえた！」）・HUDにレベル表示
- **hokora-ui**: 祠シーン・入力・表示（銅の鍵でワープ：最寄り祠を特定しもう一方の祠へテレポート、メニュー：様子を見る/扉を開ける/出る、鍵なし時メッセージ表示）

**ツール**:
- **generate_tiles**: タイルスプライト生成（独立バイナリ）
- **screenshot-common**: スクリーンショット撮影用共通ライブラリ（screenshot_app、setup_camera、screenshot_system）
- **screenshot-battle**: 戦闘画面スクリーンショット撮影バイナリ
- **screenshot-town**: 街画面スクリーンショット撮影バイナリ
- **screenshot-field**: フィールド画面スクリーンショット撮影バイナリ
- **screenshot-cave**: 洞窟画面スクリーンショット撮影バイナリ
- **screenshot-world**: ワールドマップスクリーンショット撮影バイナリ

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
| `screenshot-verifier` | UI変更時のスクリーンショット検証 |
| `task-committer` | コミット・ドキュメント整理 |
