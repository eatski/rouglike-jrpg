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
    CaveWall,
    CaveFloor,
    BossCaveWall,
    BossCaveFloor,
}

/// 地形上に配置される構造物・設置物
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Structure {
    #[default]
    None,
    Town,
    Cave,
    BossCave,
    Hokora,
    Ladder,
    WarpZone,
}

impl Structure {
    /// この構造物に歩いて到着した際のアクションを返す
    #[inline]
    pub fn tile_action(self) -> TileAction {
        match self {
            Structure::Town => TileAction::EnterTown,
            Structure::Cave => TileAction::EnterCave,
            Structure::BossCave => TileAction::EnterBossCave,
            Structure::Hokora => TileAction::EnterHokora,
            Structure::None | Structure::Ladder | Structure::WarpZone => TileAction::None,
        }
    }
}

impl Terrain {
    /// エンカウント率を返す
    #[inline]
    pub fn encounter_rate(self) -> f32 {
        match self {
            Terrain::Plains => 0.02,
            Terrain::Forest => 0.03,
            Terrain::Mountain => 0.08,
            Terrain::Sea => 0.10,
            Terrain::CaveFloor => 0.05,
            Terrain::CaveWall | Terrain::BossCaveWall | Terrain::BossCaveFloor => 0.0,
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

