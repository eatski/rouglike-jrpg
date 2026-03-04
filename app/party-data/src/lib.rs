use item::ItemKind;
use party::{
    CharacterEntry, CharacterParamTable, CombatStats, PartyMemberKind, RecruitmentPath, StatGrowth,
};
pub fn character_param_table() -> CharacterParamTable {
    CharacterParamTable::from_fn(|kind| match kind {
        PartyMemberKind::Laios => CharacterEntry {
            initial_stats: CombatStats::new(30, 8, 3, 5, 5),
            stat_growth: StatGrowth { hp: 5, mp: 1, attack: 2, defense: 1, speed: 1 },
            recruit_method: RecruitmentPath::TavernBond,
            spell_learn_table: &[
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
            ],
        },
        PartyMemberKind::Chilchuck => CharacterEntry {
            initial_stats: CombatStats::new(22, 6, 2, 9, 0),
            stat_growth: StatGrowth { hp: 3, mp: 0, attack: 2, defense: 1, speed: 2 },
            recruit_method: RecruitmentPath::GoldHire { cost: 200 },
            spell_learn_table: &[],
        },
        PartyMemberKind::Marcille => CharacterEntry {
            initial_stats: CombatStats::new(20, 2, 2, 7, 15),
            stat_growth: StatGrowth { hp: 3, mp: 3, attack: 1, defense: 1, speed: 1 },
            recruit_method: RecruitmentPath::TavernBond,
            spell_learn_table: &[
                (1, spell_data::FIRE1),
                (3, spell_data::BLAZE1),
                (5, spell_data::FIRE2),
                (7, spell_data::BLAZE2),
                (8, spell_data::SLEEP1),
                (9, spell_data::DRAIN1),
                (10, spell_data::SLEEPALL1),
            ],
        },
        PartyMemberKind::Senshi => CharacterEntry {
            initial_stats: CombatStats::new(40, 7, 6, 2, 3),
            stat_growth: StatGrowth { hp: 6, mp: 0, attack: 2, defense: 2, speed: 0 },
            recruit_method: RecruitmentPath::ItemTrade { item: ItemKind::DragonScale },
            spell_learn_table: &[
                (4, spell_data::SHIELD1),
            ],
        },
        PartyMemberKind::Falin => CharacterEntry {
            initial_stats: CombatStats::new(25, 5, 4, 4, 12),
            stat_growth: StatGrowth { hp: 4, mp: 2, attack: 1, defense: 1, speed: 1 },
            recruit_method: RecruitmentPath::TavernBond,
            spell_learn_table: &[
                (1, spell_data::HEAL1),
                (3, spell_data::HEALALL1),
                (5, spell_data::HEAL2),
                (7, spell_data::SHIELD2),
                (9, spell_data::HEALALL2),
                (10, spell_data::BARRIER2),
            ],
        },
        PartyMemberKind::Izutsumi => CharacterEntry {
            initial_stats: CombatStats::new(20, 7, 1, 10, 3),
            stat_growth: StatGrowth { hp: 3, mp: 1, attack: 2, defense: 0, speed: 2 },
            recruit_method: RecruitmentPath::ItemTrade { item: ItemKind::AncientCoin },
            spell_learn_table: &[
                (5, spell_data::FIRE1),
                (8, spell_data::BOOST1),
            ],
        },
        PartyMemberKind::Shuro => CharacterEntry {
            initial_stats: CombatStats::new(28, 10, 3, 7, 0),
            stat_growth: StatGrowth { hp: 4, mp: 0, attack: 3, defense: 1, speed: 1 },
            recruit_method: RecruitmentPath::TavernBond,
            spell_learn_table: &[],
        },
        PartyMemberKind::Namari => CharacterEntry {
            initial_stats: CombatStats::new(35, 6, 5, 3, 0),
            stat_growth: StatGrowth { hp: 5, mp: 0, attack: 2, defense: 2, speed: 0 },
            recruit_method: RecruitmentPath::GoldHire { cost: 200 },
            spell_learn_table: &[],
        },
        PartyMemberKind::Kabru => CharacterEntry {
            initial_stats: CombatStats::new(26, 7, 3, 6, 5),
            stat_growth: StatGrowth { hp: 4, mp: 1, attack: 2, defense: 1, speed: 1 },
            recruit_method: RecruitmentPath::TavernBond,
            spell_learn_table: &[
                (3, spell_data::HEAL1),
                (5, spell_data::SHIELD1),
                (6, spell_data::SIPHON1),
                (7, spell_data::BARRIER1),
                (8, spell_data::POISONALL1),
                (9, spell_data::RALLY1),
            ],
        },
        PartyMemberKind::Rinsha => CharacterEntry {
            initial_stats: CombatStats::new(24, 5, 3, 6, 8),
            stat_growth: StatGrowth { hp: 3, mp: 2, attack: 1, defense: 1, speed: 1 },
            recruit_method: RecruitmentPath::TavernBond,
            spell_learn_table: &[
                (1, spell_data::FIRE1),
                (3, spell_data::HEAL1),
                (5, spell_data::BOOST1),
                (6, spell_data::DRAIN1),
                (7, spell_data::BOOST2),
                (8, spell_data::POISON1),
                (9, spell_data::RALLY2),
            ],
        },
    })
}
