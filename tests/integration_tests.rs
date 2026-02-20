//! Bevyヘッドレス統合テスト
//!
//! MinimalPluginsを使用してウィンドウなしでECSシステムを実行し、
//! game crateのロジックとui crateのBevyシステムを結合してテストする。
//!
//! 本番（main.rs）と同じregister関数・State条件を使用し、
//! テストと本番のシステム登録の乖離を最小限に抑える。

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use battle::{default_party, Enemy};
use terrain::{Terrain, MAP_HEIGHT, MAP_WIDTH};
use world::map::generate_map;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::Duration;
use app_state::{BattleState, SceneState};
use battle_ui::{BattlePhase, BattleUIState};
use movement_ui::{
    Boat, MovementBlockedEvent, MovementLocked, OnBoat, Player,
    PlayerMovedEvent, TilePosition,
};
use app_state::{FieldMenuOpen, PartyState};
use movement_ui::{ActiveMap, MovementState, TILE_SIZE};
use world_ui::{MapModeState, SpawnPosition};

const SPAWN_X: usize = 50;
const SPAWN_Y: usize = 50;
const MAX_ANIM_FRAMES: usize = 30;

/// イベントカウンタ（テスト用）
#[derive(Resource, Default)]
struct EventCounters {
    moved_count: usize,
    blocked_count: usize,
}

/// イベントカウンタシステム
fn count_moved_events(
    mut counters: ResMut<EventCounters>,
    mut events: MessageReader<PlayerMovedEvent>,
) {
    for _ in events.read() {
        counters.moved_count += 1;
    }
}

fn count_blocked_events(
    mut counters: ResMut<EventCounters>,
    mut events: MessageReader<MovementBlockedEvent>,
) {
    for _ in events.read() {
        counters.blocked_count += 1;
    }
}

/// テスト用のBevyアプリをセットアップ（ランダムマップ）
fn setup_test_app(seed: u64) -> App {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let map_data = generate_map(&mut rng);
    let spawn_pos = map_data.spawn_position;

    setup_test_app_with_map(map_data.grid, spawn_pos.0, spawn_pos.1)
}

/// テスト用のBevyアプリをセットアップ（カスタムマップ）
///
/// 本番と同じregister関数・State条件を使用する。
fn setup_test_app_with_map(grid: Vec<Vec<Terrain>>, spawn_x: usize, spawn_y: usize) -> App {
    let mut app = App::new();

    // MinimalPluginsのみを使用（ウィンドウ、レンダリングなし）
    app.add_plugins(MinimalPlugins);

    // StatesPluginを追加（State遷移をサポート）
    app.add_plugins(bevy::state::app::StatesPlugin);

    // 時間制御を手動に設定（1フレーム=50ms）
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_relative_speed(1.0);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(50)));

    // State登録（本番と同じ）
    app.init_state::<SceneState>();   // デフォルト: Exploring
    app.init_state::<BattleState>();  // デフォルト: None

    // 必要なリソースをセットアップ
    let width = grid[0].len();
    let height = grid.len();
    let origin_x = -(width as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -(height as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    app.insert_resource(ActiveMap {
        grid,
        width,
        height,
        origin_x,
        origin_y,
    });
    app.insert_resource(SpawnPosition {
        x: spawn_x,
        y: spawn_y,
    });
    app.insert_resource(MovementState::default());
    app.insert_resource(EventCounters::default());
    app.insert_resource(MapModeState::default());
    app.init_resource::<FieldMenuOpen>();
    app.init_resource::<PartyState>();
    app.init_resource::<ButtonInput<KeyCode>>();

    // イベントを登録
    app.add_message::<PlayerMovedEvent>();
    app.add_message::<MovementBlockedEvent>();
    app.add_message::<movement_ui::PlayerArrivedEvent>();
    app.add_message::<movement_ui::TileEnteredEvent>();

    // 本番と同じシステム登録（移動コアのみ、エンカウント除外）
    world_ui::register_exploring_movement_systems(&mut app);

    // テスト専用イベントカウンタ（run_if外で常時実行）
    app.add_systems(Update, count_moved_events);
    app.add_systems(Update, count_blocked_events);

    app
}

/// プレイヤーをスポーン
fn spawn_test_player(app: &mut App) -> Entity {
    let spawn_pos = app.world().resource::<SpawnPosition>();
    let x = spawn_pos.x;
    let y = spawn_pos.y;

    app.world_mut()
        .spawn((
            Player,
            TilePosition { x, y },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .id()
}

/// 船をスポーン
fn spawn_test_boat(app: &mut App, x: usize, y: usize) -> Entity {
    app.world_mut()
        .spawn((Boat, TilePosition { x, y }, Transform::from_xyz(0.0, 0.0, 0.5)))
        .id()
}

/// 指定方向のキー入力をシミュレート
fn press_key(app: &mut App, dx: i32, dy: i32) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();

    // 全キーをリセット（pressed状態も含めて完全にクリア）
    input.reset_all();

    // 方向に応じてキーを押す
    if dx > 0 {
        input.press(KeyCode::KeyD);
    } else if dx < 0 {
        input.press(KeyCode::KeyA);
    }

    if dy > 0 {
        input.press(KeyCode::KeyW);
    } else if dy < 0 {
        input.press(KeyCode::KeyS);
    }
}

/// 単一キーを押す（他のキーはリセット）
fn press_single_key(app: &mut App, key: KeyCode) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    input.press(key);
}

/// キーをすべて離す
fn release_all_keys(app: &mut App) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
}

/// 移動アニメーションが完了するまでフレームを進める
fn wait_for_movement_complete(app: &mut App, player_entity: Entity, max_frames: usize) {
    for _ in 0..max_frames {
        app.update();

        // MovementLockedがなければ完了
        if app.world().get::<MovementLocked>(player_entity).is_none() {
            break;
        }
    }
}

/// エンティティのタイル位置を取得
fn get_tile_pos(app: &App, entity: Entity) -> (usize, usize) {
    let tp = app.world().get::<TilePosition>(entity).unwrap();
    (tp.x, tp.y)
}

/// 1回の移動を実行し、アニメーション完了まで待つ。
///
/// 最後のapp.update()でMovementStateがリセットされ、
/// 次のpress_keyでdirection_changed発火が可能になる。
fn perform_move(app: &mut App, player_entity: Entity, dx: i32, dy: i32) {
    press_key(app, dx, dy);
    app.update();
    release_all_keys(app);
    wait_for_movement_complete(app, player_entity, MAX_ANIM_FRAMES);
    app.update();
}

/// 海ベースのカスタムマップグリッドを生成
fn setup_sea_grid() -> Vec<Vec<Terrain>> {
    vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT]
}

/// イベントカウンタをリセット
fn reset_event_counters(app: &mut App) {
    let mut c = app.world_mut().resource_mut::<EventCounters>();
    c.moved_count = 0;
    c.blocked_count = 0;
}

// ============================================
// 基本移動テスト
// ============================================

#[test]
fn player_can_move_on_walkable_terrain() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y));

    reset_event_counters(&mut app);

    // 右に移動
    press_key(&mut app, 1, 0);
    app.update();
    app.update(); // イベントカウンタがメッセージを読み取るために追加フレーム

    let moved_count = app.world().resource::<EventCounters>().moved_count;
    assert!(moved_count >= 1, "PlayerMovedEvent should be emitted (got {})", moved_count);

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X + 1, SPAWN_Y), "Player should move to new position");
}

#[test]
fn player_cannot_move_into_sea() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    // grid[SPAWN_Y][SPAWN_X + 1] は海のまま

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 右に移動（海に向かう）
    press_key(&mut app, 1, 0);
    app.update();
    app.update(); // イベントカウンタがメッセージを読み取るために追加フレーム

    let blocked_count = app.world().resource::<EventCounters>().blocked_count;
    assert!(blocked_count >= 1, "MovementBlockedEvent should be emitted (got {})", blocked_count);

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y), "Player should not move into sea");
}

// ============================================
// 船移動テスト
// ============================================

#[test]
fn player_can_board_boat() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    let boat_entity = spawn_test_boat(&mut app, SPAWN_X + 1, SPAWN_Y);

    // 右に移動（船に乗る）
    perform_move(&mut app, player_entity, 1, 0);

    let world = app.world();
    let on_boat = world.get::<OnBoat>(player_entity);

    assert!(on_boat.is_some(), "Player should be on boat");
    assert_eq!(on_boat.unwrap().boat_entity, boat_entity);

    let player_pos = world.get::<TilePosition>(player_entity).unwrap();
    let boat_pos = world.get::<TilePosition>(boat_entity).unwrap();

    assert_eq!(player_pos.x, boat_pos.x);
    assert_eq!(player_pos.y, boat_pos.y);
}

#[test]
fn player_on_boat_can_move_on_sea() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    let boat_entity = spawn_test_boat(&mut app, SPAWN_X + 1, SPAWN_Y);

    // 右に移動（船に乗る）
    perform_move(&mut app, player_entity, 1, 0);

    assert!(
        app.world().get::<OnBoat>(player_entity).is_some(),
        "Player should be on boat after first move"
    );

    // さらに右に移動（海上を移動）
    perform_move(&mut app, player_entity, 1, 0);

    let final_pos = get_tile_pos(&app, player_entity);
    assert_eq!(final_pos, (SPAWN_X + 2, SPAWN_Y), "Player on boat should move on sea (got {:?})", final_pos);

    let boat_pos = app.world().get::<TilePosition>(boat_entity).unwrap();
    assert_eq!(boat_pos.x, final_pos.0);
    assert_eq!(boat_pos.y, final_pos.1);
}

#[test]
fn player_can_disembark_from_boat() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 2] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    spawn_test_boat(&mut app, SPAWN_X + 1, SPAWN_Y);

    // 右に移動（船に乗る）
    perform_move(&mut app, player_entity, 1, 0);

    // さらに右に移動（陸地に下船）
    perform_move(&mut app, player_entity, 1, 0);

    let world = app.world();
    assert!(world.get::<OnBoat>(player_entity).is_none(), "Player should have disembarked");

    let player_pos = world.get::<TilePosition>(player_entity).unwrap();
    assert_eq!(player_pos.x, SPAWN_X + 2);
    assert_eq!(player_pos.y, SPAWN_Y);
}

// ============================================
// マップ端ラップアラウンドテスト
// ============================================

#[test]
fn player_wraps_around_map_edge() {
    let mut app = setup_test_app(12345);

    // マップ端にプレイヤーを配置
    let edge_x = MAP_WIDTH - 1;
    let edge_y = MAP_HEIGHT / 2;

    // 端の位置が歩行可能か確認し、必要に応じて調整
    let (player_x, player_y) = {
        let active_map = app.world().resource::<ActiveMap>();
        let mut x = edge_x;
        let y = edge_y;

        while !active_map.grid[y][x].is_walkable() && x > 0 {
            x -= 1;
        }

        if !active_map.grid[y][x].is_walkable() {
            panic!("Could not find walkable position near edge");
        }

        (x, y)
    };

    let player_entity = app
        .world_mut()
        .spawn((
            Player,
            TilePosition {
                x: player_x,
                y: player_y,
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .id();

    // 右端が歩行可能でラップ先も歩行可能なケースをセットアップ
    let can_wrap = {
        let active_map = app.world().resource::<ActiveMap>();
        player_x == MAP_WIDTH - 1
            && active_map.grid[player_y][0].is_walkable()
    };

    if !can_wrap {
        return;
    }

    // 右に移動（マップ端を超える）
    perform_move(&mut app, player_entity, 1, 0);

    let final_pos = get_tile_pos(&app, player_entity);
    assert_eq!(final_pos.0, 0, "Player should wrap around to x=0");
    assert_eq!(final_pos.1, player_y);
}

// ============================================
// 決定性テスト
// ============================================

#[test]
fn same_seed_produces_deterministic_behavior() {
    let seed = 99999;

    let mut app1 = setup_test_app(seed);
    let player1 = spawn_test_player(&mut app1);
    let pos1_initial = get_tile_pos(&app1, player1);

    let mut app2 = setup_test_app(seed);
    let player2 = spawn_test_player(&mut app2);
    let pos2_initial = get_tile_pos(&app2, player2);

    assert_eq!(pos1_initial, pos2_initial, "Same seed should produce same spawn position");

    let map1 = app1.world().resource::<ActiveMap>();
    let map2 = app2.world().resource::<ActiveMap>();

    assert_eq!(map1.grid, map2.grid, "Same seed should produce same map");
}

// ============================================
// 地形別移動テスト
// ============================================

#[test]
fn player_can_move_on_forest() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Forest;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 右に移動（森に入る）
    press_key(&mut app, 1, 0);
    app.update();
    app.update(); // イベントカウンタがメッセージを読み取るために追加フレーム

    let moved_count = app.world().resource::<EventCounters>().moved_count;
    assert!(moved_count >= 1, "PlayerMovedEvent should be emitted when moving to forest");

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X + 1, SPAWN_Y), "Player should move onto forest");
}

#[test]
fn player_cannot_move_on_mountain() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Mountain;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 右に移動（山は通行不可）
    press_key(&mut app, 1, 0);
    app.update();

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y), "Player should not move onto mountain");
}

// ============================================
// 連続移動テスト
// ============================================

#[test]
fn player_can_move_multiple_steps() {
    let mut grid = setup_sea_grid();
    for i in 0..4 {
        grid[SPAWN_Y][SPAWN_X + i] = Terrain::Plains;
    }

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    // 3回右に移動
    for i in 0..3 {
        perform_move(&mut app, player_entity, 1, 0);

        assert_eq!(
            get_tile_pos(&app, player_entity),
            (SPAWN_X + i + 1, SPAWN_Y),
            "Player should be at step {}", i + 1
        );
    }

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X + 3, SPAWN_Y), "Player should have moved 3 steps");
}

#[test]
fn player_cannot_move_while_locked() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Plains;
    grid[SPAWN_Y + 1][SPAWN_X + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    // 右に移動開始
    press_key(&mut app, 1, 0);
    app.update();

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_some(),
        "Player should be locked during movement animation"
    );

    // ロック中に上方向のキーを押す（上は平原なので、ロック無視なら移動してしまう）
    press_key(&mut app, 0, 1);
    app.update();

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    let final_pos = get_tile_pos(&app, player_entity);
    assert_eq!(final_pos.0, SPAWN_X + 1, "X should move one step right");
    assert_eq!(final_pos.1, SPAWN_Y, "Y should not change (movement was locked)");
}

// ============================================
// バウンスアニメーションテスト
// ============================================

#[test]
fn blocked_movement_triggers_bounce_and_clears() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 右に移動（海に向かう - ブロックされる）
    press_key(&mut app, 1, 0);
    app.update();
    app.update(); // イベントカウンタがメッセージを読み取るために追加フレーム

    let blocked_count = app.world().resource::<EventCounters>().blocked_count;
    assert!(blocked_count >= 1, "MovementBlockedEvent should be emitted");

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_some(),
        "MovementLocked should be added for bounce animation"
    );

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_none(),
        "MovementLocked should be cleared after bounce"
    );

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y), "Player should not move when blocked");
}

// ============================================
// マップモードテスト
// ============================================

#[test]
fn map_mode_blocks_movement() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    // マップモードを有効化
    app.world_mut().resource_mut::<MapModeState>().enabled = true;

    reset_event_counters(&mut app);

    // 右に移動を試みる
    press_key(&mut app, 1, 0);
    app.update();

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    let counters = app.world().resource::<EventCounters>();
    assert_eq!(counters.moved_count, 0, "No PlayerMovedEvent should be emitted in map mode");
    assert_eq!(counters.blocked_count, 0, "No MovementBlockedEvent should be emitted in map mode");

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y), "Player should not move in map mode");
}

// ============================================
// PendingMove (斜め入力分解) テスト
// ============================================

#[test]
fn diagonal_input_decomposes_into_two_moves() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Plains;
    grid[SPAWN_Y + 1][SPAWN_X + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    // 斜め入力（右上）: W + D を同時押し
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.clear();
    input.press(KeyCode::KeyW); // 上
    input.press(KeyCode::KeyD); // 右
    drop(input); // borrowを解放

    app.update();

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    let pos_after_first = get_tile_pos(&app, player_entity);
    assert!(pos_after_first != (SPAWN_X, SPAWN_Y), "First diagonal move should complete");

    // PendingMoveがあれば2回目の移動が自動で実行される
    wait_for_movement_complete(&mut app, player_entity, 10);

    assert_eq!(
        get_tile_pos(&app, player_entity),
        (SPAWN_X + 1, SPAWN_Y + 1),
        "Diagonal input should decompose into two sequential moves"
    );
}

// ============================================
// イベント整合性テスト
// ============================================

#[test]
fn multiple_moves_emit_correct_event_count() {
    let mut grid = setup_sea_grid();
    for i in 0..4 {
        grid[SPAWN_Y][SPAWN_X + i] = Terrain::Plains;
    }

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 3回右に移動
    for _ in 0..3 {
        perform_move(&mut app, player_entity, 1, 0);
    }

    let counters = app.world().resource::<EventCounters>();
    assert_eq!(counters.moved_count, 3, "Should emit 3 PlayerMovedEvents");

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X + 3, SPAWN_Y), "Player should have moved 3 steps");
}

// ============================================
// 戦闘システムテスト
// ============================================

/// 戦闘用のBevyアプリをセットアップ（MinimalPlugins + State + register関数）
fn setup_battle_test_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    // StatesPluginを追加
    app.add_plugins(bevy::state::app::StatesPlugin);

    // 時間制御を手動に設定
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_relative_speed(1.0);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(50)));

    // ステートを登録
    app.init_state::<SceneState>();
    app.init_state::<BattleState>();

    // 必要なリソースをセットアップ
    app.init_resource::<ButtonInput<KeyCode>>();

    // cleanup_battle_sceneが必要とするリソース
    app.insert_resource(MovementState::default());
    app.init_resource::<PartyState>();
    app.insert_resource(ActiveMap {
        grid: vec![vec![Terrain::Plains; 1]; 1],
        width: 1,
        height: 1,
        origin_x: 0.0,
        origin_y: 0.0,
    });

    // 本番と同じシステム登録（ロジックのみ）
    battle_ui::register_battle_logic_systems(&mut app);

    app
}

/// BattleGameState + BattleUIState を init_battle_resources で生成して挿入するヘルパー
///
/// リソースを先に挿入してからBattleState::Activeに遷移する。
/// これにより、遷移時に実行されるbattle_input_systemがリソースを参照できる。
fn insert_battle_resource(app: &mut App, phase: BattlePhase) {
    // 本番と同じロジックでリソース生成
    let party = default_party();
    let enemies = vec![Enemy::slime()];
    let (game_state, mut ui_state) = battle_ui::init_battle_resources(party, enemies, None);

    // テスト用にphaseを上書き
    ui_state.phase = phase;

    // リソースを先に挿入（battle_input_systemが参照可能にする）
    app.insert_resource(game_state);
    app.insert_resource(ui_state);

    // BattleState::Activeに遷移（run_if条件を満たす）
    app.world_mut()
        .resource_mut::<NextState<BattleState>>()
        .set(BattleState::Active);
    app.update();
}

#[test]
fn battle_phase_transitions_from_command_to_exploring() {
    let mut app = setup_battle_test_app();

    // BattleResourceを挿入（CommandSelectから開始）
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert!(matches!(battle_res.phase, BattlePhase::CommandSelect { .. }));
    }

    // BattleOverまで進める
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps <= MAX_ANIM_FRAMES, "Battle should reach BattleOver within {MAX_ANIM_FRAMES} steps");

        let phase = {
            let battle_res = app.world().resource::<BattleUIState>();
            battle_res.phase.clone()
        };

        match phase {
            BattlePhase::CommandSelect { .. } => {
                // たたかうを選択（Enter）→ TargetSelectに遷移
                press_single_key(&mut app, KeyCode::Enter);
                app.update();
                release_all_keys(&mut app);
            }
            BattlePhase::SpellSelect { .. } | BattlePhase::AllyTargetSelect { .. } | BattlePhase::ItemSelect { .. } => {
                // 呪文選択や味方ターゲットが出たらキャンセルしてコマンドに戻す
                press_single_key(&mut app, KeyCode::Escape);
                app.update();
                release_all_keys(&mut app);
            }
            BattlePhase::TargetSelect { .. } => {
                // ターゲット確定（Enter）→ 次メンバーまたはターン実行
                press_single_key(&mut app, KeyCode::Enter);
                app.update();
                release_all_keys(&mut app);
            }
            BattlePhase::ShowMessage { .. } => {
                press_single_key(&mut app, KeyCode::Enter);
                app.update();
                release_all_keys(&mut app);
            }
            BattlePhase::BattleOver { .. } => {
                break;
            }
        }
    }

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert!(
            matches!(battle_res.phase, BattlePhase::BattleOver { .. }),
            "Should reach BattleOver phase"
        );
    }

    // BattleOverでEnterを押すとBattleState::Noneに遷移
    // 1フレーム目: battle_input_systemがNextStateを設定
    // 2フレーム目: StateTransitionスケジュールが状態を適用 + cleanup実行
    press_single_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let current_state = app.world().resource::<State<BattleState>>();
    assert_eq!(
        **current_state,
        BattleState::None,
        "Should transition back from battle after BattleOver"
    );
}

#[test]
fn battle_command_selection_with_keys() {
    let mut app = setup_battle_test_app();
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    // 初期状態: selected_command = 0
    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(battle_res.selected_command, 0);
    }

    // S（下）を押して選択を1に
    press_single_key(&mut app, KeyCode::KeyS);
    app.update();

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(
            battle_res.selected_command, 1,
            "Should select command 1 after pressing S"
        );
    }

    release_all_keys(&mut app);

    // W（上）を押して選択を0に戻す
    press_single_key(&mut app, KeyCode::KeyW);
    app.update();

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(
            battle_res.selected_command, 0,
            "Should select command 0 after pressing W"
        );
    }

    release_all_keys(&mut app);

    // ラップアラウンド確認: 0でW（上）を押すと3に循環
    press_single_key(&mut app, KeyCode::KeyW);
    app.update();

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(
            battle_res.selected_command, 3,
            "Should wrap to 3 (wrap around from top)"
        );
    }

    release_all_keys(&mut app);

    // ラップアラウンド確認: 3でS（下）を押すと0に循環
    press_single_key(&mut app, KeyCode::KeyS);
    app.update();

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(
            battle_res.selected_command, 0,
            "Should wrap to 0 (wrap around from bottom)"
        );
    }
}

#[test]
fn battle_flee_command_transitions_correctly() {
    let mut app = setup_battle_test_app();
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    // にげる（selected_command=3）を選択
    for _ in 0..3 {
        press_single_key(&mut app, KeyCode::KeyS);
        app.update();
        release_all_keys(&mut app);
    }

    // Enterで決定（逃走を選択）
    press_single_key(&mut app, KeyCode::Enter);
    app.update();
    release_all_keys(&mut app);

    // 逃走は50%確率。ShowMessage（にげきれた or にげられなかった）またはBattleOverに遷移
    let phase = {
        let battle_res = app.world().resource::<BattleUIState>();
        battle_res.phase.clone()
    };

    match phase {
        BattlePhase::ShowMessage { messages, .. } => {
            assert!(
                messages.iter().any(|m| m.contains("にげ")),
                "Should have flee-related message, got: {:?}",
                messages
            );
        }
        BattlePhase::BattleOver { message } => {
            assert!(
                message.contains("にげ"),
                "BattleOver message should contain flee text"
            );
        }
        _ => {
            // 逃走失敗時はCommandSelectに戻ることもある
        }
    }
}

#[test]
fn battle_cleanup_removes_movement_lock() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);

    // cleanup_battle_sceneシステムを追加（OnExit(BattleState::Active)で実行される）
    app.add_systems(OnExit(BattleState::Active), battle_ui::cleanup_battle_scene);

    let player_entity = spawn_test_player(&mut app);

    // プレイヤーに手動でMovementLockedを付与（戦闘開始前に移動中だった想定）
    app.world_mut()
        .entity_mut(player_entity)
        .insert(MovementLocked);

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_some(),
        "MovementLocked should be present before battle cleanup"
    );

    // 戦闘に入る
    app.world_mut()
        .resource_mut::<NextState<BattleState>>()
        .set(BattleState::Active);
    app.update();

    // BattleResourceを挿入（戦闘シーンセットアップの代わり）
    let party = default_party();
    let enemies = vec![Enemy::slime()];
    let (game_state, mut ui_state) = battle_ui::init_battle_resources(party, enemies, None);
    ui_state.phase = BattlePhase::CommandSelect { member_index: 0 };
    app.insert_resource(game_state);
    app.insert_resource(ui_state);

    // 戦闘終了（BattleState::Noneに戻る）→ cleanup_battle_sceneが実行される
    app.world_mut()
        .resource_mut::<NextState<BattleState>>()
        .set(BattleState::None);
    app.update();

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_none(),
        "MovementLocked should be removed after battle cleanup"
    );
}
