use party::PartyMemberKind;
use spell::SpellKind;

/// キャラ別の呪文習得テーブル: (必要レベル, 呪文) のペア
pub fn spell_learn_table(kind: PartyMemberKind) -> &'static [(u32, SpellKind)] {
    match kind {
        PartyMemberKind::Marcille => &[
            (1, SpellKind::Fire1),
            (3, SpellKind::Blaze1),
            (5, SpellKind::Fire2),
            (7, SpellKind::Blaze2),
            (9, SpellKind::Drain1),
        ],
        PartyMemberKind::Falin => &[
            (1, SpellKind::Heal1),
            (3, SpellKind::Healall1),
            (5, SpellKind::Heal2),
            (7, SpellKind::Shield2),
            (9, SpellKind::Healall2),
            (10, SpellKind::Barrier2),
        ],
        PartyMemberKind::Rinsha => &[
            (1, SpellKind::Fire1),
            (3, SpellKind::Heal1),
            (5, SpellKind::Boost1),
            (6, SpellKind::Drain1),
            (7, SpellKind::Boost2),
            (9, SpellKind::Rally2),
        ],
        PartyMemberKind::Kabru => &[
            (3, SpellKind::Heal1),
            (5, SpellKind::Shield1),
            (6, SpellKind::Siphon1),
            (7, SpellKind::Barrier1),
            (9, SpellKind::Rally1),
        ],
        PartyMemberKind::Laios => &[
            (1, SpellKind::Heal1),
            (1, SpellKind::Boost1),
            (1, SpellKind::Fire1),
            (1, SpellKind::Fire2),
            (1, SpellKind::Blaze1),
            (1, SpellKind::Blaze2),
            (1, SpellKind::Heal2),
            (1, SpellKind::Healall1),
            (1, SpellKind::Healall2),
            (1, SpellKind::Shield1),
            (1, SpellKind::Shield2),
            (1, SpellKind::Barrier1),
            (1, SpellKind::Barrier2),
            (1, SpellKind::Boost2),
            (1, SpellKind::Rally1),
            (1, SpellKind::Rally2),
            (1, SpellKind::Drain1),
            (1, SpellKind::Drain2),
            (1, SpellKind::Siphon1),
            (1, SpellKind::Siphon2),
        ],
        PartyMemberKind::Izutsumi => &[
            (5, SpellKind::Fire1),
            (8, SpellKind::Boost1),
        ],
        PartyMemberKind::Senshi => &[
            (4, SpellKind::Shield1),
        ],
        PartyMemberKind::Chilchuck
        | PartyMemberKind::Shuro
        | PartyMemberKind::Namari => &[],
    }
}

/// クラスとレベルに応じた使用可能な呪文リストを返す
pub fn available_spells(kind: PartyMemberKind, level: u32) -> Vec<SpellKind> {
    spell_learn_table(kind)
        .iter()
        .filter(|(req_level, _)| level >= *req_level)
        .map(|(_, spell)| *spell)
        .collect()
}

/// 指定レベルで新たに習得する呪文を返す
pub fn spells_learned_at_level(kind: PartyMemberKind, level: u32) -> Vec<SpellKind> {
    spell_learn_table(kind)
        .iter()
        .filter(|(req_level, _)| *req_level == level)
        .map(|(_, spell)| *spell)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn laios_has_all_20_spells_at_level_1() {
        let spells = available_spells(PartyMemberKind::Laios, 1);
        assert_eq!(spells.len(), 20);
    }

    #[test]
    fn marcille_learns_fire1_at_level_1() {
        let spells = available_spells(PartyMemberKind::Marcille, 1);
        assert_eq!(spells, vec![SpellKind::Fire1]);
    }

    #[test]
    fn marcille_learns_blaze1_at_level_3() {
        let spells = available_spells(PartyMemberKind::Marcille, 3);
        assert_eq!(spells, vec![SpellKind::Fire1, SpellKind::Blaze1]);
    }

    #[test]
    fn marcille_learns_fire2_at_level_5() {
        let spells = available_spells(PartyMemberKind::Marcille, 5);
        assert_eq!(
            spells,
            vec![SpellKind::Fire1, SpellKind::Blaze1, SpellKind::Fire2]
        );
    }

    #[test]
    fn falin_learns_heal1_at_level_1() {
        let spells = available_spells(PartyMemberKind::Falin, 1);
        assert_eq!(spells, vec![SpellKind::Heal1]);
    }

    #[test]
    fn falin_max_spells() {
        let spells = available_spells(PartyMemberKind::Falin, 10);
        assert_eq!(spells.len(), 6);
    }

    #[test]
    fn spells_learned_at_specific_level() {
        assert_eq!(
            spells_learned_at_level(PartyMemberKind::Marcille, 1),
            vec![SpellKind::Fire1]
        );
        assert_eq!(
            spells_learned_at_level(PartyMemberKind::Marcille, 3),
            vec![SpellKind::Blaze1]
        );
        assert!(spells_learned_at_level(PartyMemberKind::Marcille, 2).is_empty());
    }

    #[test]
    fn senshi_learns_shield1_at_level_4() {
        let spells = available_spells(PartyMemberKind::Senshi, 4);
        assert_eq!(spells, vec![SpellKind::Shield1]);
    }

    #[test]
    fn no_spell_characters() {
        assert!(available_spells(PartyMemberKind::Chilchuck, 99).is_empty());
        assert!(available_spells(PartyMemberKind::Shuro, 99).is_empty());
        assert!(available_spells(PartyMemberKind::Namari, 99).is_empty());
    }

    #[test]
    fn marcille_learns_drain1_at_level_9() {
        let spells = available_spells(PartyMemberKind::Marcille, 9);
        assert!(spells.contains(&SpellKind::Drain1));
    }

    #[test]
    fn rinsha_learns_drain1_at_level_6() {
        let spells = available_spells(PartyMemberKind::Rinsha, 6);
        assert!(spells.contains(&SpellKind::Drain1));
        assert!(!available_spells(PartyMemberKind::Rinsha, 5).contains(&SpellKind::Drain1));
    }

    #[test]
    fn kabru_learns_siphon1_at_level_6() {
        let spells = available_spells(PartyMemberKind::Kabru, 6);
        assert!(spells.contains(&SpellKind::Siphon1));
        assert!(!available_spells(PartyMemberKind::Kabru, 5).contains(&SpellKind::Siphon1));
    }
}
