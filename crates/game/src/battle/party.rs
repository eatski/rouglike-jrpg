use super::stats::CombatStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyMemberKind {
    Hero,
    Mage,
    Priest,
}

impl PartyMemberKind {
    pub fn name(self) -> &'static str {
        match self {
            PartyMemberKind::Hero => "勇者",
            PartyMemberKind::Mage => "魔法使い",
            PartyMemberKind::Priest => "僧侶",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PartyMember {
    pub kind: PartyMemberKind,
    pub stats: CombatStats,
}

impl PartyMember {
    pub fn hero() -> Self {
        Self {
            kind: PartyMemberKind::Hero,
            stats: CombatStats::new(30, 8, 3, 5, 5),
        }
    }

    pub fn mage() -> Self {
        Self {
            kind: PartyMemberKind::Mage,
            stats: CombatStats::new(20, 10, 2, 7, 15),
        }
    }

    pub fn priest() -> Self {
        Self {
            kind: PartyMemberKind::Priest,
            stats: CombatStats::new(25, 5, 4, 4, 12),
        }
    }
}

pub fn default_party() -> Vec<PartyMember> {
    vec![
        PartyMember::hero(),
        PartyMember::mage(),
        PartyMember::priest(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_party_has_three_members() {
        let party = default_party();
        assert_eq!(party.len(), 3);
        assert_eq!(party[0].kind, PartyMemberKind::Hero);
        assert_eq!(party[1].kind, PartyMemberKind::Mage);
        assert_eq!(party[2].kind, PartyMemberKind::Priest);
    }

    #[test]
    fn hero_stats() {
        let hero = PartyMember::hero();
        assert_eq!(hero.stats.max_hp, 30);
        assert_eq!(hero.stats.attack, 8);
        assert_eq!(hero.stats.defense, 3);
        assert_eq!(hero.stats.speed, 5);
    }

    #[test]
    fn mage_stats() {
        let mage = PartyMember::mage();
        assert_eq!(mage.stats.max_hp, 20);
        assert_eq!(mage.stats.attack, 10);
        assert_eq!(mage.stats.defense, 2);
        assert_eq!(mage.stats.speed, 7);
    }

    #[test]
    fn priest_stats() {
        let priest = PartyMember::priest();
        assert_eq!(priest.stats.max_hp, 25);
        assert_eq!(priest.stats.attack, 5);
        assert_eq!(priest.stats.defense, 4);
        assert_eq!(priest.stats.speed, 4);
    }
}
