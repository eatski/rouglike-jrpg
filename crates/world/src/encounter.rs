use terrain::Terrain;

/// エンカウント判定
///
/// 船乗車中はエンカウントなし。地形ごとの確率:
/// - 草原: 2%
/// - 森: 3%
/// - 山: 8%
/// - 海: 10%
pub fn should_encounter(terrain: Terrain, on_boat: bool, random_value: f32) -> bool {
    if on_boat {
        return false;
    }
    let rate = encounter_rate(terrain);
    random_value < rate
}

fn encounter_rate(terrain: Terrain) -> f32 {
    match terrain {
        Terrain::Plains => 0.02,
        Terrain::Forest => 0.03,
        Terrain::Mountain => 0.08,
        Terrain::Sea => 0.10,
        Terrain::Town => 0.0,
        Terrain::Cave => 0.0,
    }
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
