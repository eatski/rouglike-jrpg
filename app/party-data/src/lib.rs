use item::ItemKind;
use party::{
    CharacterEntry, CharacterParamTable, CombatStats, PartyMemberKind, RecruitmentPath, StatGrowth,
};
use spell_data::SpellKind;

pub fn character_param_table() -> CharacterParamTable {
    CharacterParamTable::from_fn(|kind| match kind {
        PartyMemberKind::Laios => CharacterEntry {
            initial_stats: CombatStats::new(30, 8, 3, 5, 5),
            stat_growth: StatGrowth { hp: 5, mp: 1, attack: 2, defense: 1, speed: 1 },
            recruit_method: RecruitmentPath::TavernBond,
            spell_learn_table: const { &[
                (1, SpellKind::Heal1.entry()), (1, SpellKind::Boost1.entry()),
                (1, SpellKind::Fire1.entry()), (1, SpellKind::Fire2.entry()),
                (1, SpellKind::Blaze1.entry()), (1, SpellKind::Blaze2.entry()),
                (1, SpellKind::Heal2.entry()), (1, SpellKind::Healall1.entry()),
                (1, SpellKind::Healall2.entry()), (1, SpellKind::Shield1.entry()),
                (1, SpellKind::Shield2.entry()), (1, SpellKind::Barrier1.entry()),
                (1, SpellKind::Barrier2.entry()), (1, SpellKind::Boost2.entry()),
                (1, SpellKind::Rally1.entry()), (1, SpellKind::Rally2.entry()),
                (1, SpellKind::Drain1.entry()), (1, SpellKind::Drain2.entry()),
                (1, SpellKind::Siphon1.entry()), (1, SpellKind::Siphon2.entry()),
                (1, SpellKind::Sleep1.entry()), (1, SpellKind::Sleepall1.entry()),
                (1, SpellKind::Poison1.entry()), (1, SpellKind::Poisonall1.entry()),
            ] },
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
            spell_learn_table: const { &[
                (1, SpellKind::Fire1.entry()),
                (3, SpellKind::Blaze1.entry()),
                (5, SpellKind::Fire2.entry()),
                (7, SpellKind::Blaze2.entry()),
                (8, SpellKind::Sleep1.entry()),
                (9, SpellKind::Drain1.entry()),
                (10, SpellKind::Sleepall1.entry()),
            ] },
        },
        PartyMemberKind::Senshi => CharacterEntry {
            initial_stats: CombatStats::new(40, 7, 6, 2, 3),
            stat_growth: StatGrowth { hp: 6, mp: 0, attack: 2, defense: 2, speed: 0 },
            recruit_method: RecruitmentPath::ItemTrade { item: ItemKind::DragonScale },
            spell_learn_table: const { &[
                (4, SpellKind::Shield1.entry()),
            ] },
        },
        PartyMemberKind::Falin => CharacterEntry {
            initial_stats: CombatStats::new(25, 5, 4, 4, 12),
            stat_growth: StatGrowth { hp: 4, mp: 2, attack: 1, defense: 1, speed: 1 },
            recruit_method: RecruitmentPath::TavernBond,
            spell_learn_table: const { &[
                (1, SpellKind::Heal1.entry()),
                (3, SpellKind::Healall1.entry()),
                (5, SpellKind::Heal2.entry()),
                (7, SpellKind::Shield2.entry()),
                (9, SpellKind::Healall2.entry()),
                (10, SpellKind::Barrier2.entry()),
            ] },
        },
        PartyMemberKind::Izutsumi => CharacterEntry {
            initial_stats: CombatStats::new(20, 7, 1, 10, 3),
            stat_growth: StatGrowth { hp: 3, mp: 1, attack: 2, defense: 0, speed: 2 },
            recruit_method: RecruitmentPath::ItemTrade { item: ItemKind::AncientCoin },
            spell_learn_table: const { &[
                (5, SpellKind::Fire1.entry()),
                (8, SpellKind::Boost1.entry()),
            ] },
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
            spell_learn_table: const { &[
                (3, SpellKind::Heal1.entry()),
                (5, SpellKind::Shield1.entry()),
                (6, SpellKind::Siphon1.entry()),
                (7, SpellKind::Barrier1.entry()),
                (8, SpellKind::Poisonall1.entry()),
                (9, SpellKind::Rally1.entry()),
            ] },
        },
        PartyMemberKind::Rinsha => CharacterEntry {
            initial_stats: CombatStats::new(24, 5, 3, 6, 8),
            stat_growth: StatGrowth { hp: 3, mp: 2, attack: 1, defense: 1, speed: 1 },
            recruit_method: RecruitmentPath::TavernBond,
            spell_learn_table: const { &[
                (1, SpellKind::Fire1.entry()),
                (3, SpellKind::Heal1.entry()),
                (5, SpellKind::Boost1.entry()),
                (6, SpellKind::Drain1.entry()),
                (7, SpellKind::Boost2.entry()),
                (8, SpellKind::Poison1.entry()),
                (9, SpellKind::Rally2.entry()),
            ] },
        },
    })
}
