#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Plains,
    Mountain,
    Forest,
    Sea,
    Town,
}

impl Terrain {
    /// 徒歩で通行可能かどうかを判定
    ///
    /// 海以外は全て通行可能。
    #[inline]
    pub fn is_walkable(self) -> bool {
        self != Terrain::Sea
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
    }

    #[test]
    fn is_walkable_returns_false_for_sea() {
        assert!(!Terrain::Sea.is_walkable());
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
    }
}
