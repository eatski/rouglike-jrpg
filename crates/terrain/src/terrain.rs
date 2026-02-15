pub const MAP_WIDTH: usize = 150;
pub const MAP_HEIGHT: usize = 150;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileAction {
    EnterTown,
    EnterCave,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Plains,
    Mountain,
    Forest,
    Sea,
    Town,
    Cave,
    CaveWall,
    CaveFloor,
    WarpZone,
    Ladder,
}

impl Terrain {
    /// このタイルに歩いて到着した際のアクションを返す
    #[inline]
    pub fn tile_action(self) -> TileAction {
        match self {
            Terrain::Town => TileAction::EnterTown,
            Terrain::Cave => TileAction::EnterCave,
            _ => TileAction::None,
        }
    }

    /// エンカウント率を返す
    #[inline]
    pub fn encounter_rate(self) -> f32 {
        match self {
            Terrain::Plains => 0.02,
            Terrain::Forest => 0.03,
            Terrain::Mountain => 0.08,
            Terrain::Sea => 0.10,
            Terrain::CaveFloor => 0.05,
            Terrain::Town | Terrain::Cave | Terrain::CaveWall | Terrain::WarpZone | Terrain::Ladder => 0.0,
        }
    }

    /// 徒歩で通行可能かどうかを判定
    #[inline]
    pub fn is_walkable(self) -> bool {
        !matches!(self, Terrain::Sea | Terrain::CaveWall)
    }

    /// 船で航行可能かどうかを判定
    ///
    /// 海のみ航行可能。
    #[inline]
    pub fn is_navigable(self) -> bool {
        self == Terrain::Sea
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_walkable_returns_true_for_land() {
        assert!(Terrain::Plains.is_walkable());
        assert!(Terrain::Mountain.is_walkable());
        assert!(Terrain::Forest.is_walkable());
        assert!(Terrain::Town.is_walkable());
        assert!(Terrain::Cave.is_walkable());
        assert!(Terrain::CaveFloor.is_walkable());
        assert!(Terrain::WarpZone.is_walkable());
        assert!(Terrain::Ladder.is_walkable());
    }

    #[test]
    fn is_walkable_returns_false_for_impassable() {
        assert!(!Terrain::Sea.is_walkable());
        assert!(!Terrain::CaveWall.is_walkable());
    }

    #[test]
    fn is_navigable_returns_true_for_sea() {
        assert!(Terrain::Sea.is_navigable());
    }

    #[test]
    fn is_navigable_returns_false_for_land() {
        assert!(!Terrain::Plains.is_navigable());
        assert!(!Terrain::Mountain.is_navigable());
        assert!(!Terrain::Forest.is_navigable());
        assert!(!Terrain::Town.is_navigable());
        assert!(!Terrain::Cave.is_navigable());
    }

    #[test]
    fn tile_action_returns_enter_town_for_town() {
        assert_eq!(Terrain::Town.tile_action(), TileAction::EnterTown);
    }

    #[test]
    fn tile_action_returns_enter_cave_for_cave() {
        assert_eq!(Terrain::Cave.tile_action(), TileAction::EnterCave);
    }

    #[test]
    fn tile_action_returns_none_for_other_terrains() {
        assert_eq!(Terrain::Plains.tile_action(), TileAction::None);
        assert_eq!(Terrain::Mountain.tile_action(), TileAction::None);
        assert_eq!(Terrain::Forest.tile_action(), TileAction::None);
        assert_eq!(Terrain::Sea.tile_action(), TileAction::None);
    }

    #[test]
    fn encounter_rate_cave_floor() {
        assert!(Terrain::CaveFloor.encounter_rate() > 0.0);
    }

    #[test]
    fn encounter_rate_zero_for_walls_and_towns() {
        assert_eq!(Terrain::Town.encounter_rate(), 0.0);
        assert_eq!(Terrain::CaveWall.encounter_rate(), 0.0);
        assert_eq!(Terrain::WarpZone.encounter_rate(), 0.0);
        assert_eq!(Terrain::Ladder.encounter_rate(), 0.0);
    }
}
