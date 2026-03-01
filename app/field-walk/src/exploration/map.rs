use super::visibility::{calculate_visible_tiles, TileVisibility};

/// 探索マップ（Fog of War）を管理する構造体
pub struct ExplorationMap {
    width: usize,
    height: usize,
    tiles: Vec<Vec<TileVisibility>>,
    /// 現在Visibleなタイルの座標（差分更新用）
    current_visible: Vec<(usize, usize)>,
}

impl ExplorationMap {
    /// 新しいExplorationMapを作成
    ///
    /// # Arguments
    /// * `width` - マップの幅
    /// * `height` - マップの高さ
    ///
    /// # Returns
    /// すべてのタイルがUnexploredで初期化されたExplorationMap
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![vec![TileVisibility::default(); width]; height],
            current_visible: Vec::new(),
        }
    }

    /// プレイヤーの視界を更新
    ///
    /// 以前Visibleだったタイルは自動的にExploredに変更され、
    /// 新しい視界範囲内のタイルがVisibleに設定される
    ///
    /// # Arguments
    /// * `player_x` - プレイヤーのx座標
    /// * `player_y` - プレイヤーのy座標
    /// * `view_radius` - 視界半径
    pub fn update_visibility(&mut self, player_x: usize, player_y: usize, view_radius: usize) {
        // 以前のVisibleタイルのみをExploredに変更（差分更新）
        for (x, y) in self.current_visible.drain(..) {
            self.tiles[y][x] = TileVisibility::Explored;
        }

        // 新しい視界範囲を計算して Visible に設定
        let visible_tiles =
            calculate_visible_tiles(player_x, player_y, view_radius, self.width, self.height);

        for (x, y) in visible_tiles {
            self.tiles[y][x] = TileVisibility::Visible;
            self.current_visible.push((x, y));
        }
    }

    /// 指定座標の可視状態を取得
    ///
    /// # Arguments
    /// * `x` - x座標
    /// * `y` - y座標
    ///
    /// # Returns
    /// 座標が範囲内ならその可視状態、範囲外ならNone
    pub fn get(&self, x: usize, y: usize) -> Option<TileVisibility> {
        if y < self.height && x < self.width {
            Some(self.tiles[y][x])
        } else {
            None
        }
    }

    /// 探索済み（Explored または Visible）のタイル座標を取得
    ///
    /// # Returns
    /// 探索済みタイルの(x, y)座標のイテレータ
    pub fn get_explored_tiles(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.tiles
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, tile)| {
                    if *tile == TileVisibility::Explored || *tile == TileVisibility::Visible {
                        Some((x, y))
                    } else {
                        None
                    }
                })
            })
    }

    /// マップの幅を取得
    pub fn width(&self) -> usize {
        self.width
    }

    /// マップの高さを取得
    pub fn height(&self) -> usize {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_unexplored_map() {
        let map = ExplorationMap::new(10, 10);

        // すべてのタイルがUnexploredで初期化されているか確認
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(map.get(x, y), Some(TileVisibility::Unexplored));
            }
        }
    }

    #[test]
    fn test_get_returns_none_for_out_of_bounds() {
        let map = ExplorationMap::new(10, 10);

        assert_eq!(map.get(10, 5), None);
        assert_eq!(map.get(5, 10), None);
        assert_eq!(map.get(100, 100), None);
    }

    #[test]
    fn test_update_visibility_marks_tiles_visible() {
        let mut map = ExplorationMap::new(10, 10);

        // (5, 5) から半径1の視界を更新
        map.update_visibility(5, 5, 1);

        // 中心タイルがVisibleになっているか確認
        assert_eq!(map.get(5, 5), Some(TileVisibility::Visible));

        // 周囲のタイルがVisibleになっているか確認
        assert_eq!(map.get(4, 4), Some(TileVisibility::Visible));
        assert_eq!(map.get(5, 4), Some(TileVisibility::Visible));
        assert_eq!(map.get(6, 4), Some(TileVisibility::Visible));
        assert_eq!(map.get(4, 5), Some(TileVisibility::Visible));
        assert_eq!(map.get(6, 5), Some(TileVisibility::Visible));
        assert_eq!(map.get(4, 6), Some(TileVisibility::Visible));
        assert_eq!(map.get(5, 6), Some(TileVisibility::Visible));
        assert_eq!(map.get(6, 6), Some(TileVisibility::Visible));

        // 視界外のタイルはUnexploredのまま
        assert_eq!(map.get(0, 0), Some(TileVisibility::Unexplored));
        assert_eq!(map.get(9, 9), Some(TileVisibility::Unexplored));
    }

    #[test]
    fn test_update_visibility_transitions_visible_to_explored() {
        let mut map = ExplorationMap::new(10, 10);

        // 最初の位置 (5, 5) から視界更新
        map.update_visibility(5, 5, 1);
        assert_eq!(map.get(5, 5), Some(TileVisibility::Visible));

        // 別の位置 (7, 7) に移動して視界更新
        map.update_visibility(7, 7, 1);

        // 以前の中心 (5, 5) はExploredになっているはず
        assert_eq!(map.get(5, 5), Some(TileVisibility::Explored));

        // 新しい中心 (7, 7) はVisibleになっているはず
        assert_eq!(map.get(7, 7), Some(TileVisibility::Visible));
    }

    #[test]
    fn test_update_visibility_explored_remains_explored() {
        let mut map = ExplorationMap::new(10, 10);

        // (5, 5) で視界更新
        map.update_visibility(5, 5, 1);

        // (7, 7) に移動（(5, 5)は視界外）
        map.update_visibility(7, 7, 1);

        // (5, 5) はExploredになっているはず
        assert_eq!(map.get(5, 5), Some(TileVisibility::Explored));

        // さらに (9, 9) に移動
        map.update_visibility(9, 9, 1);

        // (5, 5) はExploredのまま（Unexploredに戻らない）
        assert_eq!(map.get(5, 5), Some(TileVisibility::Explored));
    }

    #[test]
    fn test_update_visibility_can_revisit_explored_tiles() {
        let mut map = ExplorationMap::new(10, 10);

        // (5, 5) で視界更新
        map.update_visibility(5, 5, 1);
        assert_eq!(map.get(5, 5), Some(TileVisibility::Visible));

        // (7, 7) に移動
        map.update_visibility(7, 7, 1);
        assert_eq!(map.get(5, 5), Some(TileVisibility::Explored));

        // (5, 5) に戻る
        map.update_visibility(5, 5, 1);

        // 再度Visibleになる
        assert_eq!(map.get(5, 5), Some(TileVisibility::Visible));
    }

    #[test]
    fn test_get_explored_tiles_empty_initially() {
        let map = ExplorationMap::new(10, 10);

        let explored: Vec<_> = map.get_explored_tiles().collect();
        assert_eq!(explored.len(), 0);
    }

    #[test]
    fn test_get_explored_tiles_includes_visible() {
        let mut map = ExplorationMap::new(10, 10);

        // (5, 5) から半径1の視界
        map.update_visibility(5, 5, 1);

        let explored: Vec<_> = map.get_explored_tiles().collect();

        // 3x3 = 9タイルが探索済み（Visible）
        assert_eq!(explored.len(), 9);

        // 中心タイルが含まれているか確認
        assert!(explored.contains(&(5, 5)));
    }

    #[test]
    fn test_get_explored_tiles_includes_explored_and_visible() {
        let mut map = ExplorationMap::new(10, 10);

        // (5, 5) で視界更新
        map.update_visibility(5, 5, 1);

        // (7, 7) に移動
        map.update_visibility(7, 7, 1);

        let explored: Vec<_> = map.get_explored_tiles().collect();

        // 以前の視界 (5, 5 周辺) + 新しい視界 (7, 7 周辺)
        // 一部重複があるので正確な数は計算が必要だが、少なくとも9タイル以上
        assert!(explored.len() >= 9);

        // (5, 5) はExploredとして含まれる
        assert!(explored.contains(&(5, 5)));

        // (7, 7) はVisibleとして含まれる
        assert!(explored.contains(&(7, 7)));
    }

    #[test]
    fn test_update_visibility_with_wrap() {
        let mut map = ExplorationMap::new(10, 10);

        // 左上角 (0, 0) から半径1の視界（トーラスラップ）
        map.update_visibility(0, 0, 1);

        // ラップして右下角もVisibleになるはず
        assert_eq!(map.get(9, 9), Some(TileVisibility::Visible));
        assert_eq!(map.get(0, 9), Some(TileVisibility::Visible));
        assert_eq!(map.get(9, 0), Some(TileVisibility::Visible));
    }

    #[test]
    fn test_visibility_state_transitions() {
        let mut map = ExplorationMap::new(10, 10);

        // Unexplored → Visible
        assert_eq!(map.get(5, 5), Some(TileVisibility::Unexplored));
        map.update_visibility(5, 5, 1);
        assert_eq!(map.get(5, 5), Some(TileVisibility::Visible));

        // Visible → Explored
        map.update_visibility(7, 7, 1);
        assert_eq!(map.get(5, 5), Some(TileVisibility::Explored));

        // Explored → Visible（再訪問）
        map.update_visibility(5, 5, 1);
        assert_eq!(map.get(5, 5), Some(TileVisibility::Visible));
    }

    #[test]
    fn test_crossing_edge_maintains_visible_count() {
        let mut map = ExplorationMap::new(150, 150);

        // Visibleタイルの数をカウントするヘルパー
        fn count_visible(map: &ExplorationMap) -> usize {
            let mut count = 0;
            for y in 0..map.height() {
                for x in 0..map.width() {
                    if map.get(x, y) == Some(TileVisibility::Visible) {
                        count += 1;
                    }
                }
            }
            count
        }

        // 中央から開始（半径4 = 9x9 = 81タイル）
        map.update_visibility(75, 75, 4);
        assert_eq!(count_visible(&map), 81);

        // 端に近づく
        map.update_visibility(147, 75, 4);
        assert_eq!(count_visible(&map), 81);

        // 端を越える（x=148→149→0）
        map.update_visibility(148, 75, 4);
        assert_eq!(count_visible(&map), 81);

        map.update_visibility(149, 75, 4);
        assert_eq!(count_visible(&map), 81);

        map.update_visibility(0, 75, 4);
        assert_eq!(count_visible(&map), 81);

        // さらに進む
        map.update_visibility(1, 75, 4);
        assert_eq!(count_visible(&map), 81);
    }
}
