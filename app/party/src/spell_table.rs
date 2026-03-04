use crate::character_table::CharacterParamTable;
use crate::party::PartyMemberKind;
use spell::SpellEntry;

/// クラスとレベルに応じた使用可能な呪文リストを返す
pub fn available_spells(kind: PartyMemberKind, level: u32, table: &CharacterParamTable) -> Vec<SpellEntry> {
    table.spell_learn_table(kind)
        .iter()
        .filter(|(req_level, _)| level >= *req_level)
        .map(|(_, spell)| *spell)
        .collect()
}

/// 指定レベルで新たに習得する呪文を返す
pub fn spells_learned_at_level(kind: PartyMemberKind, level: u32, table: &CharacterParamTable) -> Vec<SpellEntry> {
    table.spell_learn_table(kind)
        .iter()
        .filter(|(req_level, _)| *req_level == level)
        .map(|(_, spell)| *spell)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character_table::{CharacterEntry, CharacterParamTable};
    use crate::party::RecruitmentPath;
    use crate::stats::{CombatStats, StatGrowth};
    

    fn char_table() -> CharacterParamTable {
        CharacterParamTable::from_fn(|kind| match kind {
            PartyMemberKind::Marcille => CharacterEntry {
                initial_stats: CombatStats::new(20, 2, 2, 7, 15),
                stat_growth: StatGrowth { hp: 3, mp: 3, attack: 1, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: const { &[
                    (1, spell_data::FIRE1),
                    (3, spell_data::BLAZE1),
                    (5, spell_data::FIRE2),
                    (7, spell_data::BLAZE2),
                    (8, spell_data::SLEEP1),
                    (9, spell_data::DRAIN1),
                    (10, spell_data::SLEEPALL1),
                ] },
            },
            PartyMemberKind::Falin => CharacterEntry {
                initial_stats: CombatStats::new(25, 5, 4, 4, 12),
                stat_growth: StatGrowth { hp: 4, mp: 2, attack: 1, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: const { &[
                    (1, spell_data::HEAL1),
                    (3, spell_data::HEALALL1),
                    (5, spell_data::HEAL2),
                    (7, spell_data::SHIELD2),
                    (9, spell_data::HEALALL2),
                    (10, spell_data::BARRIER2),
                ] },
            },
            PartyMemberKind::Rinsha => CharacterEntry {
                initial_stats: CombatStats::new(24, 5, 3, 6, 8),
                stat_growth: StatGrowth { hp: 3, mp: 2, attack: 1, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: const { &[
                    (1, spell_data::FIRE1),
                    (3, spell_data::HEAL1),
                    (5, spell_data::BOOST1),
                    (6, spell_data::DRAIN1),
                    (7, spell_data::BOOST2),
                    (8, spell_data::POISON1),
                    (9, spell_data::RALLY2),
                ] },
            },
            PartyMemberKind::Kabru => CharacterEntry {
                initial_stats: CombatStats::new(26, 7, 3, 6, 5),
                stat_growth: StatGrowth { hp: 4, mp: 1, attack: 2, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: const { &[
                    (3, spell_data::HEAL1),
                    (5, spell_data::SHIELD1),
                    (6, spell_data::SIPHON1),
                    (7, spell_data::BARRIER1),
                    (8, spell_data::POISONALL1),
                    (9, spell_data::RALLY1),
                ] },
            },
            PartyMemberKind::Laios => CharacterEntry {
                initial_stats: CombatStats::new(30, 8, 3, 5, 5),
                stat_growth: StatGrowth { hp: 5, mp: 1, attack: 2, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: const { &[
                    (1, spell_data::HEAL1), (1, spell_data::BOOST1),
                    (1, spell_data::FIRE1), (1, spell_data::FIRE2),
                    (1, spell_data::BLAZE1), (1, spell_data::BLAZE2),
                    (1, spell_data::HEAL2), (1, spell_data::HEALALL1),
                    (1, spell_data::HEALALL2), (1, spell_data::SHIELD1),
                    (1, spell_data::SHIELD2), (1, spell_data::BARRIER1),
                    (1, spell_data::BARRIER2), (1, spell_data::BOOST2),
                    (1, spell_data::RALLY1), (1, spell_data::RALLY2),
                    (1, spell_data::DRAIN1), (1, spell_data::DRAIN2),
                    (1, spell_data::SIPHON1), (1, spell_data::SIPHON2),
                    (1, spell_data::SLEEP1), (1, spell_data::SLEEPALL1),
                    (1, spell_data::POISON1), (1, spell_data::POISONALL1),
                ] },
            },
            PartyMemberKind::Izutsumi => CharacterEntry {
                initial_stats: CombatStats::new(20, 7, 1, 10, 3),
                stat_growth: StatGrowth { hp: 3, mp: 1, attack: 2, defense: 0, speed: 2 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: const { &[
                    (5, spell_data::FIRE1),
                    (8, spell_data::BOOST1),
                ] },
            },
            PartyMemberKind::Senshi => CharacterEntry {
                initial_stats: CombatStats::new(40, 7, 6, 2, 3),
                stat_growth: StatGrowth { hp: 6, mp: 0, attack: 2, defense: 2, speed: 0 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: const { &[
                    (4, spell_data::SHIELD1),
                ] },
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
        assert_eq!(spells, vec![spell_data::FIRE1]);
    }

    #[test]
    fn marcille_learns_blaze1_at_level_3() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Marcille, 3, &table);
        assert_eq!(spells, vec![spell_data::FIRE1, spell_data::BLAZE1]);
    }

    #[test]
    fn marcille_learns_fire2_at_level_5() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Marcille, 5, &table);
        assert_eq!(
            spells,
            vec![spell_data::FIRE1, spell_data::BLAZE1, spell_data::FIRE2]
        );
    }

    #[test]
    fn falin_learns_heal1_at_level_1() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Falin, 1, &table);
        assert_eq!(spells, vec![spell_data::HEAL1]);
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
            vec![spell_data::FIRE1]
        );
        assert_eq!(
            spells_learned_at_level(PartyMemberKind::Marcille, 3, &table),
            vec![spell_data::BLAZE1]
        );
        assert!(spells_learned_at_level(PartyMemberKind::Marcille, 2, &table).is_empty());
    }

    #[test]
    fn senshi_learns_shield1_at_level_4() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Senshi, 4, &table);
        assert_eq!(spells, vec![spell_data::SHIELD1]);
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
        assert!(spells.contains(&spell_data::DRAIN1));
    }

    #[test]
    fn rinsha_learns_drain1_at_level_6() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Rinsha, 6, &table);
        assert!(spells.contains(&spell_data::DRAIN1));
        assert!(!available_spells(PartyMemberKind::Rinsha, 5, &table).contains(&spell_data::DRAIN1));
    }

    #[test]
    fn kabru_learns_siphon1_at_level_6() {
        let table = char_table();
        let spells = available_spells(PartyMemberKind::Kabru, 6, &table);
        assert!(spells.contains(&spell_data::SIPHON1));
        assert!(!available_spells(PartyMemberKind::Kabru, 5, &table).contains(&spell_data::SIPHON1));
    }
}
