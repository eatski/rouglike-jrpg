use crate::map::Terrain;

/// 地形に応じたエンカウント判定
///
/// - 草原: 10%
/// - 森: 15%
/// - 山/海: エンカウントなし
pub fn should_encounter(terrain: Terrain, random_value: f32) -> bool {
    let rate = encounter_rate(terrain);
    random_value < rate
}

fn encounter_rate(terrain: Terrain) -> f32 {
    match terrain {
        Terrain::Plains => 0.10,
        Terrain::Forest => 0.15,
        Terrain::Mountain | Terrain::Sea => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plains_encounter_at_boundary() {
        assert!(should_encounter(Terrain::Plains, 0.09));
        assert!(!should_encounter(Terrain::Plains, 0.10));
        assert!(!should_encounter(Terrain::Plains, 0.11));
    }

    #[test]
    fn forest_encounter_at_boundary() {
        assert!(should_encounter(Terrain::Forest, 0.14));
        assert!(!should_encounter(Terrain::Forest, 0.15));
        assert!(!should_encounter(Terrain::Forest, 0.16));
    }

    #[test]
    fn mountain_never_encounters() {
        assert!(!should_encounter(Terrain::Mountain, 0.0));
        assert!(!should_encounter(Terrain::Mountain, 0.5));
    }

    #[test]
    fn sea_never_encounters() {
        assert!(!should_encounter(Terrain::Sea, 0.0));
        assert!(!should_encounter(Terrain::Sea, 0.5));
    }
}
