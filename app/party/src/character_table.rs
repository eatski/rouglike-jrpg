use crate::party::{PartyMemberKind, RecruitmentPath};
use crate::stats::{CombatStats, StatGrowth};
use spell::SpellKind;

/// 1キャラクターのデータエントリ
pub struct CharacterEntry {
    pub initial_stats: CombatStats,
    pub stat_growth: StatGrowth,
    pub recruit_method: RecruitmentPath,
    pub spell_learn_table: &'static [(u32, SpellKind)],
}

/// 全キャラのデータテーブル
pub struct CharacterParamTable {
    entries: Vec<CharacterEntry>,
}

const ALL_KINDS: [PartyMemberKind; 10] = [
    PartyMemberKind::Laios,
    PartyMemberKind::Chilchuck,
    PartyMemberKind::Marcille,
    PartyMemberKind::Senshi,
    PartyMemberKind::Falin,
    PartyMemberKind::Izutsumi,
    PartyMemberKind::Shuro,
    PartyMemberKind::Namari,
    PartyMemberKind::Kabru,
    PartyMemberKind::Rinsha,
];

fn kind_index(kind: PartyMemberKind) -> usize {
    ALL_KINDS
        .iter()
        .position(|&k| k == kind)
        .expect("unknown PartyMemberKind")
}

pub fn all_kinds() -> &'static [PartyMemberKind] {
    &ALL_KINDS
}

impl CharacterParamTable {
    pub fn from_fn(f: impl Fn(PartyMemberKind) -> CharacterEntry) -> Self {
        let entries = ALL_KINDS.iter().map(|&kind| f(kind)).collect();
        Self { entries }
    }

    pub fn get(&self, kind: PartyMemberKind) -> &CharacterEntry {
        &self.entries[kind_index(kind)]
    }

    pub fn initial_stats(&self, kind: PartyMemberKind) -> CombatStats {
        self.entries[kind_index(kind)].initial_stats.clone()
    }

    pub fn stat_growth(&self, kind: PartyMemberKind) -> &StatGrowth {
        &self.entries[kind_index(kind)].stat_growth
    }

    pub fn recruit_method(&self, kind: PartyMemberKind) -> &RecruitmentPath {
        &self.entries[kind_index(kind)].recruit_method
    }

    pub fn spell_learn_table(&self, kind: PartyMemberKind) -> &'static [(u32, SpellKind)] {
        self.entries[kind_index(kind)].spell_learn_table
    }
}
