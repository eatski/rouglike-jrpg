use terrain::Terrain;

/// エンカウント判定
///
/// 船乗車中はエンカウントなし。地形ごとの確率はTerrain::encounter_rate()で定義。
pub fn should_encounter(terrain: Terrain, on_boat: bool, random_value: f32) -> bool {
    if on_boat {
        return false;
    }
    random_value < terrain.encounter_rate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plains_encounter_at_boundary() {
        assert!(should_encounter(Terrain::Plains, false, 0.01));
        assert!(!should_encounter(Terrain::Plains, false, 0.02));
        assert!(!should_encounter(Terrain::Plains, false, 0.03));
    }

    #[test]
    fn forest_encounter_at_boundary() {
        assert!(should_encounter(Terrain::Forest, false, 0.02));
        assert!(!should_encounter(Terrain::Forest, false, 0.03));
        assert!(!should_encounter(Terrain::Forest, false, 0.04));
    }

    #[test]
    fn mountain_encounter_at_boundary() {
        assert!(should_encounter(Terrain::Mountain, false, 0.07));
        assert!(!should_encounter(Terrain::Mountain, false, 0.08));
        assert!(!should_encounter(Terrain::Mountain, false, 0.09));
    }

    #[test]
    fn sea_encounter_at_boundary() {
        assert!(should_encounter(Terrain::Sea, false, 0.09));
        assert!(!should_encounter(Terrain::Sea, false, 0.10));
        assert!(!should_encounter(Terrain::Sea, false, 0.11));
    }

    #[test]
    fn town_never_encounters() {
        assert!(!should_encounter(Terrain::Town, false, 0.0));
        assert!(!should_encounter(Terrain::Town, false, 0.5));
    }

    #[test]
    fn on_boat_never_encounters() {
        assert!(!should_encounter(Terrain::Plains, true, 0.0));
        assert!(!should_encounter(Terrain::Forest, true, 0.0));
        assert!(!should_encounter(Terrain::Sea, true, 0.0));
    }
}
