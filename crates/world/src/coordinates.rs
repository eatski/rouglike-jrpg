//! 座標計算ユーティリティ
//!
//! terrainの座標関数を再エクスポートする。

pub use terrain::coordinates::{
    is_diagonal_movement, orthogonal_neighbors, wrap_coordinate, wrap_position, Direction,
    ORTHOGONAL_DIRECTIONS,
};
