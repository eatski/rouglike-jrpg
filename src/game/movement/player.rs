use bevy::prelude::*;

use crate::game::map::{MapData, Terrain, MAP_HEIGHT, MAP_WIDTH};

use super::events::{MovementBlockedEvent, PlayerMovedEvent};

/// 移動試行の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    /// 移動成功: 新しい位置
    Moved { new_x: usize, new_y: usize },
    /// 移動失敗: 海に阻まれた
    Blocked,
}

/// 移動を試みる（純粋関数）
///
/// タイル座標と方向を受け取り、移動結果を返す。
/// マップ端ではラップアラウンドする。
pub fn try_move(
    current_x: usize,
    current_y: usize,
    dx: i32,
    dy: i32,
    grid: &[Vec<Terrain>],
) -> MoveResult {
    let new_x = ((current_x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
    let new_y = ((current_y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;

    if grid[new_y][new_x] == Terrain::Sea {
        MoveResult::Blocked
    } else {
        MoveResult::Moved { new_x, new_y }
    }
}

/// 地形が通行可能かどうかを判定する
pub fn is_passable(terrain: Terrain) -> bool {
    terrain != Terrain::Sea
}

#[derive(Component)]
pub struct Player;

/// 移動処理中かどうかを示すマーカーコンポーネント（UI側で設定）
#[derive(Component)]
pub struct MovementLocked;

#[derive(Component)]
pub struct TilePosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Resource)]
pub struct SpawnPosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Resource)]
pub struct MovementState {
    pub timer: Timer,
    pub initial_delay: Timer,
    pub is_repeating: bool,
    pub last_direction: (i32, i32),
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.08, TimerMode::Repeating),
            initial_delay: Timer::from_seconds(0.2, TimerMode::Once),
            is_repeating: false,
            last_direction: (0, 0),
        }
    }
}

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map_data: Res<MapData>,
    mut move_state: ResMut<MovementState>,
    mut query: Query<
        (Entity, &mut TilePosition, Option<&MovementLocked>),
        With<Player>,
    >,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
) {
    let Ok((entity, mut tile_pos, locked)) = query.single_mut() else {
        return;
    };

    // 移動ロック中は入力を無視
    if locked.is_some() {
        return;
    }

    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        dy = 1;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dy = -1;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dx = -1;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dx = 1;
    }

    let current_direction = (dx, dy);

    // 方向キーが押されていない場合はリセット
    if dx == 0 && dy == 0 {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = (0, 0);
        return;
    }

    // 方向が変わったか判定（新しい入力として扱う）
    let direction_changed = current_direction != move_state.last_direction;

    let should_move = if direction_changed {
        // 方向変更時は即座に移動、タイマーリセット
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = current_direction;
        true
    } else if move_state.is_repeating {
        // リピート中は通常のタイマーで移動
        move_state.timer.tick(time.delta());
        move_state.timer.just_finished()
    } else {
        // 初回遅延を待つ
        move_state.initial_delay.tick(time.delta());
        if move_state.initial_delay.just_finished() {
            move_state.is_repeating = true;
            move_state.timer.reset();
            true
        } else {
            false
        }
    };

    if should_move {
        match try_move(tile_pos.x, tile_pos.y, dx, dy, &map_data.grid) {
            MoveResult::Blocked => {
                blocked_events.write(MovementBlockedEvent {
                    entity,
                    direction: (dx, dy),
                });
            }
            MoveResult::Moved { new_x, new_y } => {
                tile_pos.x = new_x;
                tile_pos.y = new_y;
                moved_events.write(PlayerMovedEvent {
                    entity,
                    direction: (dx, dy),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用の小さなグリッドを作成するヘルパー
    fn create_test_grid(width: usize, height: usize, default: Terrain) -> Vec<Vec<Terrain>> {
        vec![vec![default; width]; height]
    }

    // ============================================
    // try_move のテスト
    // ============================================

    #[test]
    fn try_move_succeeds_on_plains() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Plains;

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_move_succeeds_on_forest() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Forest;

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_move_succeeds_on_mountain() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Mountain;

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_move_blocked_by_sea() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        // grid[5][6] はデフォルトでSea

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_move_wraps_around_right_edge() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][MAP_WIDTH - 1] = Terrain::Plains;
        grid[5][0] = Terrain::Plains; // ラップ先

        let result = try_move(MAP_WIDTH - 1, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 0, new_y: 5 });
    }

    #[test]
    fn try_move_wraps_around_left_edge() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][0] = Terrain::Plains;
        grid[5][MAP_WIDTH - 1] = Terrain::Plains; // ラップ先

        let result = try_move(0, 5, -1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: MAP_WIDTH - 1, new_y: 5 });
    }

    #[test]
    fn try_move_wraps_around_top_edge() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[MAP_HEIGHT - 1][5] = Terrain::Plains;
        grid[0][5] = Terrain::Plains; // ラップ先

        let result = try_move(5, MAP_HEIGHT - 1, 0, 1, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: 0 });
    }

    #[test]
    fn try_move_wraps_around_bottom_edge() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[0][5] = Terrain::Plains;
        grid[MAP_HEIGHT - 1][5] = Terrain::Plains; // ラップ先

        let result = try_move(5, 0, 0, -1, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: MAP_HEIGHT - 1 });
    }

    #[test]
    fn try_move_diagonal_movement() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[6][6] = Terrain::Plains;

        let result = try_move(5, 5, 1, 1, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 6 });
    }

    #[test]
    fn try_move_no_movement() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;

        let result = try_move(5, 5, 0, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: 5 });
    }

    // ============================================
    // is_passable のテスト
    // ============================================

    #[test]
    fn is_passable_plains() {
        assert!(is_passable(Terrain::Plains));
    }

    #[test]
    fn is_passable_forest() {
        assert!(is_passable(Terrain::Forest));
    }

    #[test]
    fn is_passable_mountain() {
        assert!(is_passable(Terrain::Mountain));
    }

    #[test]
    fn is_passable_sea_returns_false() {
        assert!(!is_passable(Terrain::Sea));
    }
}
