use item::{Equipment, Inventory, ItemKind};
use crate::character_table::CharacterParamTable;
use crate::stats::CombatStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyMemberKind {
    Laios,
    Chilchuck,
    Marcille,
    Senshi,
    Falin,
    Izutsumi,
    Shuro,
    Namari,
    Kabru,
    Rinsha,
}

impl PartyMemberKind {
    pub fn name(self) -> &'static str {
        match self {
            PartyMemberKind::Laios => "ライオス",
            PartyMemberKind::Chilchuck => "チルチャック",
            PartyMemberKind::Marcille => "マルシル",
            PartyMemberKind::Senshi => "センシ",
            PartyMemberKind::Falin => "ファリン",
            PartyMemberKind::Izutsumi => "イヅツミ",
            PartyMemberKind::Shuro => "シュロー",
            PartyMemberKind::Namari => "ナマリ",
            PartyMemberKind::Kabru => "カブルー",
            PartyMemberKind::Rinsha => "リンシャ",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PartyMember {
    pub kind: PartyMemberKind,
    pub level: u32,
    pub exp: u32,
    pub stats: CombatStats,
    pub inventory: Inventory,
    pub equipment: Equipment,
}

/// 次のレベルに必要な累計経験値
pub fn exp_to_next_level(level: u32) -> u32 {
    // Lv1→2: 10, Lv2→3: 25, Lv3→4: 50, ...
    // 式: level^2 * 5 + level * 5
    level * level * 5 + level * 5
}

impl PartyMember {
    /// 戦闘結果を永続状態に反映する（kindは変更しない）
    pub fn sync_from_battle(&mut self, battle_member: &PartyMember) {
        self.stats = battle_member.stats.clone();
        self.level = battle_member.level;
        self.exp = battle_member.exp;
        self.inventory = battle_member.inventory.clone();
        self.equipment = battle_member.equipment.clone();
    }

    /// 装備込みの実効攻撃力
    pub fn effective_attack(&self, item_table: &item::ItemParamTable) -> i32 {
        self.stats.attack + self.equipment.attack_bonus(item_table)
    }

    /// 経験値を獲得し、レベルアップがあれば回数を返す
    pub fn gain_exp(&mut self, amount: u32, table: &CharacterParamTable) -> u32 {
        self.exp += amount;
        let mut level_ups = 0;
        while self.exp >= exp_to_next_level(self.level) {
            self.level += 1;
            level_ups += 1;
            let growth = table.stat_growth(self.kind);
            self.stats.apply_growth(growth);
        }
        level_ups
    }

    pub fn from_kind(kind: PartyMemberKind, table: &CharacterParamTable) -> Self {
        Self {
            kind,
            level: 1,
            exp: 0,
            stats: table.initial_stats(kind),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }
}

pub fn default_party(table: &CharacterParamTable) -> Vec<PartyMember> {
    vec![
        PartyMember::from_kind(PartyMemberKind::Laios, table),
        PartyMember::from_kind(PartyMemberKind::Marcille, table),
        PartyMember::from_kind(PartyMemberKind::Falin, table),
    ]
}

/// ゲーム開始時の初期パーティ（ライオスのみ）
pub fn initial_party(table: &CharacterParamTable) -> Vec<PartyMember> {
    vec![PartyMember::from_kind(PartyMemberKind::Laios, table)]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecruitmentPath {
    /// 居酒屋2回で仲間になる
    TavernBond,
    /// 金を払って雇う
    GoldHire { cost: u32 },
    /// アイテムと引き換えに仲間になる
    ItemTrade { item: ItemKind },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecruitmentStatus {
    /// まだ出会っていない
    Undiscovered,
    /// 知り合いになった（最初の街で会話済み）
    Acquaintance,
    /// 正式にパーティ加入済み
    Recruited,
}

#[derive(Debug, Clone)]
pub struct RecruitCandidate {
    pub kind: PartyMemberKind,
    pub status: RecruitmentStatus,
}

impl RecruitCandidate {
    pub fn new(kind: PartyMemberKind) -> Self {
        Self {
            kind,
            status: RecruitmentStatus::Undiscovered,
        }
    }
}

/// 全仲間候補リストを返す（ライオス以外）
pub fn default_candidates() -> Vec<RecruitCandidate> {
    vec![
        RecruitCandidate::new(PartyMemberKind::Chilchuck),
        RecruitCandidate::new(PartyMemberKind::Marcille),
        RecruitCandidate::new(PartyMemberKind::Senshi),
        RecruitCandidate::new(PartyMemberKind::Falin),
        RecruitCandidate::new(PartyMemberKind::Izutsumi),
        RecruitCandidate::new(PartyMemberKind::Shuro),
        RecruitCandidate::new(PartyMemberKind::Namari),
        RecruitCandidate::new(PartyMemberKind::Kabru),
        RecruitCandidate::new(PartyMemberKind::Rinsha),
    ]
}

#[derive(Debug, PartialEq, Eq)]
pub enum TalkResult {
    /// 初対面 → 知り合いになった
    BecameAcquaintance,
    /// 2回目 → 仲間に加入
    Recruited,
    /// 既に仲間
    AlreadyRecruited,
}

/// 仲間候補に話しかける
pub fn talk_to_candidate(candidate: &mut RecruitCandidate) -> TalkResult {
    match candidate.status {
        RecruitmentStatus::Undiscovered => {
            candidate.status = RecruitmentStatus::Acquaintance;
            TalkResult::BecameAcquaintance
        }
        RecruitmentStatus::Acquaintance => {
            candidate.status = RecruitmentStatus::Recruited;
            TalkResult::Recruited
        }
        RecruitmentStatus::Recruited => TalkResult::AlreadyRecruited,
    }
}

/// パーティ全体（メンバー+ふくろ）で指定アイテムを持っているか
pub fn has_item(members: &[PartyMember], bag: &Inventory, item: ItemKind) -> bool {
    members.iter().any(|m| m.inventory.count(item) > 0) || bag.count(item) > 0
}

/// メンバー→ふくろの順で指定アイテムを1つ消費する。成功したらtrue
pub fn consume_item(members: &mut [PartyMember], bag: &mut Inventory, item: ItemKind) -> bool {
    for member in members.iter_mut() {
        if member.inventory.remove_item(item) {
            return true;
        }
    }
    bag.remove_item(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character_table::{CharacterEntry, CharacterParamTable};
    use crate::stats::StatGrowth;

    fn char_table() -> CharacterParamTable {
        CharacterParamTable::from_fn(|kind| match kind {
            PartyMemberKind::Laios => CharacterEntry {
                initial_stats: CombatStats::new(30, 8, 3, 5, 5),
                stat_growth: StatGrowth { hp: 5, mp: 1, attack: 2, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[],
            },
            _ => CharacterEntry {
                initial_stats: CombatStats::new(20, 5, 2, 5, 5),
                stat_growth: StatGrowth { hp: 3, mp: 1, attack: 1, defense: 1, speed: 1 },
                recruit_method: RecruitmentPath::TavernBond,
                spell_learn_table: &[],
            },
        })
    }

    #[test]
    fn exp_to_next_level_values() {
        assert_eq!(exp_to_next_level(1), 10);  // Lv1→2
        assert_eq!(exp_to_next_level(2), 30);  // Lv2→3
        assert_eq!(exp_to_next_level(3), 60);  // Lv3→4
    }

    #[test]
    fn gain_exp_levels_up() {
        let table = char_table();
        let mut laios = PartyMember::from_kind(PartyMemberKind::Laios, &table);
        let level_ups = laios.gain_exp(10, &table); // ちょうどLv2に
        assert_eq!(level_ups, 1);
        assert_eq!(laios.level, 2);
        assert_eq!(laios.exp, 10);
        assert_eq!(laios.stats.max_hp, 35); // +5
        assert_eq!(laios.stats.attack, 10); // +2
    }

    #[test]
    fn gain_exp_multiple_level_ups() {
        let table = char_table();
        let mut laios = PartyMember::from_kind(PartyMemberKind::Laios, &table);
        let level_ups = laios.gain_exp(60, &table); // Lv1→2(10) → Lv2→3(30) → Lv3→4(60)
        assert_eq!(level_ups, 3);
        assert_eq!(laios.level, 4);
    }

    #[test]
    fn gain_exp_no_level_up() {
        let table = char_table();
        let mut laios = PartyMember::from_kind(PartyMemberKind::Laios, &table);
        let level_ups = laios.gain_exp(5, &table);
        assert_eq!(level_ups, 0);
        assert_eq!(laios.level, 1);
        assert_eq!(laios.exp, 5);
    }

    #[test]
    fn talk_to_candidate_transitions() {
        let mut candidate = RecruitCandidate::new(PartyMemberKind::Marcille);

        let result = talk_to_candidate(&mut candidate);
        assert_eq!(result, TalkResult::BecameAcquaintance);
        assert_eq!(candidate.status, RecruitmentStatus::Acquaintance);

        let result = talk_to_candidate(&mut candidate);
        assert_eq!(result, TalkResult::Recruited);
        assert_eq!(candidate.status, RecruitmentStatus::Recruited);

        let result = talk_to_candidate(&mut candidate);
        assert_eq!(result, TalkResult::AlreadyRecruited);
    }

    #[test]
    fn has_item_finds_in_member_inventory() {
        let mut members = vec![PartyMember::laios()];
        let bag = Inventory::with_capacity(10);
        members[0].inventory.add(ItemKind::Herb, 1);
        assert!(has_item(&members, &bag, ItemKind::Herb));
        assert!(!has_item(&members, &bag, ItemKind::HighHerb));
    }

    #[test]
    fn has_item_finds_in_bag() {
        let members = vec![PartyMember::laios()];
        let mut bag = Inventory::with_capacity(10);
        bag.add(ItemKind::HighHerb, 1);
        assert!(has_item(&members, &bag, ItemKind::HighHerb));
    }

    #[test]
    fn consume_item_prefers_member_over_bag() {
        let mut members = vec![PartyMember::laios()];
        let mut bag = Inventory::with_capacity(10);
        members[0].inventory.add(ItemKind::Herb, 1);
        bag.add(ItemKind::Herb, 1);
        assert!(consume_item(&mut members, &mut bag, ItemKind::Herb));
        // メンバーから先に消費される
        assert_eq!(members[0].inventory.count(ItemKind::Herb), 0);
        assert_eq!(bag.count(ItemKind::Herb), 1);
    }

    #[test]
    fn consume_item_falls_back_to_bag() {
        let mut members = vec![PartyMember::laios()];
        let mut bag = Inventory::with_capacity(10);
        bag.add(ItemKind::Herb, 1);
        assert!(consume_item(&mut members, &mut bag, ItemKind::Herb));
        assert_eq!(bag.count(ItemKind::Herb), 0);
    }

    #[test]
    fn consume_item_returns_false_when_absent() {
        let mut members = vec![PartyMember::laios()];
        let mut bag = Inventory::with_capacity(10);
        assert!(!consume_item(&mut members, &mut bag, ItemKind::Herb));
    }

    #[test]
    fn effective_attack_without_weapon() {
        let table = char_table();
        let item_table = item_data::item_param_table();
        let laios = PartyMember::from_kind(PartyMemberKind::Laios, &table);
        assert_eq!(laios.effective_attack(&item_table), laios.stats.attack);
    }

    #[test]
    fn effective_attack_with_weapon() {
        use item::WeaponKind;
        let table = char_table();
        let item_table = item_data::item_param_table();
        let mut laios = PartyMember::from_kind(PartyMemberKind::Laios, &table);
        laios.equipment.equip_weapon(WeaponKind::IronSword);
        assert_eq!(laios.effective_attack(&item_table), laios.stats.attack + 5);
    }
}
