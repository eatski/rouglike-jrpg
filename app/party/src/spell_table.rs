use crate::character_table::CharacterParamTable;
use crate::party::PartyMemberKind;
use spell::SpellEntry;

/// クラスとレベルに応じた使用可能な呪文リストを返す
pub fn available_spells(kind: PartyMemberKind, level: u32, table: &CharacterParamTable) -> Vec<SpellEntry> {
    table.spell_learn_table(kind)
        .iter()
        .filter(|(req_level, _)| level >= *req_level)
        .map(|(_, spell)| spell.entry())
        .collect()
}

/// 指定レベルで新たに習得する呪文を返す
pub fn spells_learned_at_level(kind: PartyMemberKind, level: u32, table: &CharacterParamTable) -> Vec<SpellEntry> {
    table.spell_learn_table(kind)
        .iter()
        .filter(|(req_level, _)| *req_level == level)
        .map(|(_, spell)| spell.entry())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character_table::{CharacterEntry, CharacterParamTable};
    use crate::party::RecruitmentPath;
    use crate::stats::{CombatStats, StatGrowth};
    use spell_data::SpellKind;

    fn char_table() -> CharacterParamTable {
        CharacterParamTable::from_fn(|kind| match kind {
            PartyMemberKind::Marcille => CharacterEntry {
                initial_stats: CombatStats::new(20, 2, 2, 7, 15),
                stat_growth: StatGrowth { hp: 3, mp: 3, attack: 1, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[
                    (1, SpellKind::Fire1),
                    (3, SpellKind::Blaze1),
                    (5, SpellKind::Fire2),
                    (7, SpellKind::Blaze2),
                    (8, SpellKind::Sleep1),
                    (9, SpellKind::Drain1),
                    (10, SpellKind::Sleepall1),
                ],
            },
            PartyMemberKind::Falin => CharacterEntry {
                initial_stats: CombatStats::new(25, 5, 4, 4, 12),
                stat_growth: StatGrowth { hp: 4, mp: 2, attack: 1, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[
                    (1, SpellKind::Heal1),
                    (3, SpellKind::Healall1),
                    (5, SpellKind::Heal2),
                    (7, SpellKind::Shield2),
                    (9, SpellKind::Healall2),
                    (10, SpellKind::Barrier2),
                ],
            },
            PartyMemberKind::Rinsha => CharacterEntry {
                initial_stats: CombatStats::new(24, 5, 3, 6, 8),
                stat_growth: StatGrowth { hp: 3, mp: 2, attack: 1, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[
                    (1, SpellKind::Fire1),
                    (3, SpellKind::Heal1),
                    (5, SpellKind::Boost1),
                    (6, SpellKind::Drain1),
                    (7, SpellKind::Boost2),
                    (8, SpellKind::Poison1),
                    (9, SpellKind::Rally2),
                ],
            },
            PartyMemberKind::Kabru => CharacterEntry {
                initial_stats: CombatStats::new(26, 7, 3, 6, 5),
                stat_growth: StatGrowth { hp: 4, mp: 1, attack: 2, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[
                    (3, SpellKind::Heal1),
                    (5, SpellKind::Shield1),
                    (6, SpellKind::Siphon1),
                    (7, SpellKind::Barrier1),
                    (8, SpellKind::Poisonall1),
                    (9, SpellKind::Rally1),
                ],
            },
            PartyMemberKind::Laios => CharacterEntry {
                initial_stats: CombatStats::new(30, 8, 3, 5, 5),
                stat_growth: StatGrowth { hp: 5, mp: 1, attack: 2, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[
                    (1, SpellKind::Heal1), (1, SpellKind::Boost1),
                    (1, SpellKind::Fire1), (1, SpellKind::Fire2),
                    (1, SpellKind::Blaze1), (1, SpellKind::Blaze2),
                    (1, SpellKind::Heal2), (1, SpellKind::Healall1),
                    (1, SpellKind::Healall2), (1, SpellKind::Shield1),
                    (1, SpellKind::Shield2), (1, SpellKind::Barrier1),
                    (1, SpellKind::Barrier2), (1, SpellKind::Boost2),
                    (1, SpellKind::Rally1), (1, SpellKind::Rally2),
                    (1, SpellKind::Drain1), (1, SpellKind::Drain2),
                    (1, SpellKind::Siphon1), (1, SpellKind::Siphon2),
                    (1, SpellKind::Sleep1), (1, SpellKind::Sleepall1),
                    (1, SpellKind::Poison1), (1, SpellKind::Poisonall1),
                ],
            },
            PartyMemberKind::Izutsumi => CharacterEntry {
                initial_stats: CombatStats::new(20, 7, 1, 10, 3),
                stat_growth: StatGrowth { hp: 3, mp: 1, attack: 2, defense: 0, speed: 2 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[
                    (5, SpellKind::Fire1),
                    (8, SpellKind::Boost1),
                ],
            },
            PartyMemberKind::Senshi => CharacterEntry {
                initial_stats: CombatStats::new(40, 7, 6, 2, 3),
                stat_growth: StatGrowth { hp: 6, mp: 0, attack: 2, defense: 2, speed: 0 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[
                    (4, SpellKind::Shield1),
                ],
            },
            _ => CharacterEntry {
                initial_stats: CombatStats::new(20, 5, 2, 5, 0),
                stat_growth: StatGrowth { hp: 3, mp: 0, attack: 1, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[],
            },
        })
    }

    #[test]
    fn laios_has_all_24_spells_at_level_1() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Laios, 1, &table);
        assert_eq!(spells.len(), 24);
    }

    #[test]
    fn marcille_learns_fire1_at_level_1() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Marcille, 1, &table);
        assert_eq!(spells, vec![SpellKind::Fire1.entry()]);
    }

    #[test]
    fn marcille_learns_blaze1_at_level_3() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Marcille, 3, &table);
        assert_eq!(spells, vec![SpellKind::Fire1.entry(), SpellKind::Blaze1.entry()]);
    }

    #[test]
    fn marcille_learns_fire2_at_level_5() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Marcille, 5, &table);
        assert_eq!(
            spells,
            vec![SpellKind::Fire1.entry(), SpellKind::Blaze1.entry(), SpellKind::Fire2.entry()]
        );
    }

    #[test]
    fn falin_learns_heal1_at_level_1() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Falin, 1, &table);
        assert_eq!(spells, vec![SpellKind::Heal1.entry()]);
    }

    #[test]
    fn falin_max_spells() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Falin, 10, &table);
        assert_eq!(spells.len(), 6);
    }

    #[test]
    fn spells_learned_at_specific_level() {
        let table = char_table();
        assert_eq!(
            spells_learned_at_level(PartyMemberKind::Marcille, 1, &table),
            vec![SpellKind::Fire1.entry()]
        );
        assert_eq!(
            spells_learned_at_level(PartyMemberKind::Marcille, 3, &table),
            vec![SpellKind::Blaze1.entry()]
        );
        assert!(spells_learned_at_level(PartyMemberKind::Marcille, 2, &table).is_empty());
    }

    #[test]
    fn senshi_learns_shield1_at_level_4() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Senshi, 4, &table);
        assert_eq!(spells, vec![SpellKind::Shield1.entry()]);
    }

    #[test]
    fn no_spell_characters() {
        let table = char_table();
        assert!(available_spells(PartyMemberKind::Chilchuck, 99, &table).is_empty());
        assert!(available_spells(PartyMemberKind::Shuro, 99, &table).is_empty());
        assert!(available_spells(PartyMemberKind::Namari, 99, &table).is_empty());
    }

    #[test]
    fn marcille_learns_drain1_at_level_9() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Marcille, 9, &table);
        assert!(spells.contains(&SpellKind::Drain1.entry()));
    }

    #[test]
    fn rinsha_learns_drain1_at_level_6() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Rinsha, 6, &table);
        assert!(spells.contains(&SpellKind::Drain1.entry()));
        assert!(!available_spells(PartyMemberKind::Rinsha, 5, &table).contains(&SpellKind::Drain1.entry()));
    }

    #[test]
    fn kabru_learns_siphon1_at_level_6() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Kabru, 6, &table);
        assert!(spells.contains(&SpellKind::Siphon1.entry()));
        assert!(!available_spells(PartyMemberKind::Kabru, 5, &table).contains(&SpellKind::Siphon1.entry()));
    }
}
