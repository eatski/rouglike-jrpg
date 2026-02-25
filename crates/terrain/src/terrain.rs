pub const MAP_WIDTH: usize = 200;
pub const MAP_HEIGHT: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileAction {
    EnterTown,
    EnterCave,
    EnterBossCave,
    EnterHokora,
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
    Hokora,
    BossCave,
    BossCaveWall,
    BossCaveFloor,
}

impl Terrain {
    /// このタイルに歩いて到着した際のアクションを返す
    #[inline]
    pub fn tile_action(self) -> TileAction {
        match self {
            Terrain::Town => TileAction::EnterTown,
            Terrain::Cave => TileAction::EnterCave,
            Terrain::BossCave => TileAction::EnterBossCave,
            Terrain::Hokora => TileAction::EnterHokora,
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
            Terrain::Town | Terrain::Cave | Terrain::CaveWall | Terrain::WarpZone | Terrain::Ladder | Terrain::Hokora
            | Terrain::BossCave | Terrain::BossCaveWall | Terrain::BossCaveFloor => 0.0,
        }
    }

    /// 徒歩で通行可能かどうかを判定
    #[inline]
    pub fn is_walkable(self) -> bool {
        !matches!(self, Terrain::Sea | Terrain::Mountain | Terrain::CaveWall | Terrain::BossCaveWall)
    }

    /// 船で航行可能かどうかを判定
    ///
    /// 海のみ航行可能。
    #[inline]
    pub fn is_navigable(self) -> bool {
        self == Terrain::Sea
    }
}

