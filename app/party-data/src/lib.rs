use item::ItemKind;
use party::{
    CharacterEntry, CharacterParamTable, CombatStats, PartyMemberKind, RecruitmentPath, StatGrowth,
};
use spell::SpellKind;

pub fn character_param_table() -> CharacterParamTable {
    CharacterParamTable::from_fn(|kind| match kind {
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
                (1, SpellKind::Fire1),
                (3, SpellKind::Blaze1),
                (5, SpellKind::Fire2),
                (7, SpellKind::Blaze2),
                (8, SpellKind::Sleep1),
                (9, SpellKind::Drain1),
                (10, SpellKind::Sleepall1),
            ],
        },
        PartyMemberKind::Senshi => CharacterEntry {
            initial_stats: CombatStats::new(40, 7, 6, 2, 3),
            stat_growth: StatGrowth { hp: 6, mp: 0, attack: 2, defense: 2, speed: 0 },
            recruit_method: RecruitmentPath::ItemTrade { item: ItemKind::DragonScale },
            spell_learn_table: &[
                (4, SpellKind::Shield1),
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
        PartyMemberKind::Izutsumi => CharacterEntry {
            initial_stats: CombatStats::new(20, 7, 1, 10, 3),
            stat_growth: StatGrowth { hp: 3, mp: 1, attack: 2, defense: 0, speed: 2 },
            recruit_method: RecruitmentPath::ItemTrade { item: ItemKind::AncientCoin },
            spell_learn_table: &[
                (5, SpellKind::Fire1),
                (8, SpellKind::Boost1),
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
                (3, SpellKind::Heal1),
                (5, SpellKind::Shield1),
                (6, SpellKind::Siphon1),
                (7, SpellKind::Barrier1),
                (8, SpellKind::Poisonall1),
                (9, SpellKind::Rally1),
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
    })
}
