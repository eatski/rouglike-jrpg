use crate::equipment::Equipment;
use crate::item::Inventory;
use crate::stats::{CombatStats, StatGrowth};

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
    pub fn effective_attack(&self) -> i32 {
        self.stats.attack + self.equipment.attack_bonus()
    }

    /// 経験値を獲得し、レベルアップがあれば回数を返す
    pub fn gain_exp(&mut self, amount: u32) -> u32 {
        self.exp += amount;
        let mut level_ups = 0;
        while self.exp >= exp_to_next_level(self.level) {
            self.level += 1;
            level_ups += 1;
            let growth = self.kind.stat_growth();
            self.stats.apply_growth(&growth);
        }
        level_ups
    }

    pub fn laios() -> Self {
        Self {
            kind: PartyMemberKind::Laios,
            level: 1,
            exp: 0,
            stats: CombatStats::new(30, 8, 3, 5, 5),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn chilchuck() -> Self {
        Self {
            kind: PartyMemberKind::Chilchuck,
            level: 1,
            exp: 0,
            stats: CombatStats::new(22, 6, 2, 9, 0),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn marcille() -> Self {
        Self {
            kind: PartyMemberKind::Marcille,
            level: 1,
            exp: 0,
            stats: CombatStats::new(20, 2, 2, 7, 15),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn senshi() -> Self {
        Self {
            kind: PartyMemberKind::Senshi,
            level: 1,
            exp: 0,
            stats: CombatStats::new(40, 7, 6, 2, 3),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn falin() -> Self {
        Self {
            kind: PartyMemberKind::Falin,
            level: 1,
            exp: 0,
            stats: CombatStats::new(25, 5, 4, 4, 12),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn izutsumi() -> Self {
        Self {
            kind: PartyMemberKind::Izutsumi,
            level: 1,
            exp: 0,
            stats: CombatStats::new(20, 7, 1, 10, 3),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn shuro() -> Self {
        Self {
            kind: PartyMemberKind::Shuro,
            level: 1,
            exp: 0,
            stats: CombatStats::new(28, 10, 3, 7, 0),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn namari() -> Self {
        Self {
            kind: PartyMemberKind::Namari,
            level: 1,
            exp: 0,
            stats: CombatStats::new(35, 6, 5, 3, 0),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn kabru() -> Self {
        Self {
            kind: PartyMemberKind::Kabru,
            level: 1,
            exp: 0,
            stats: CombatStats::new(26, 7, 3, 6, 5),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn rinsha() -> Self {
        Self {
            kind: PartyMemberKind::Rinsha,
            level: 1,
            exp: 0,
            stats: CombatStats::new(24, 5, 3, 6, 8),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn from_kind(kind: PartyMemberKind) -> Self {
        match kind {
            PartyMemberKind::Laios => Self::laios(),
            PartyMemberKind::Chilchuck => Self::chilchuck(),
            PartyMemberKind::Marcille => Self::marcille(),
            PartyMemberKind::Senshi => Self::senshi(),
            PartyMemberKind::Falin => Self::falin(),
            PartyMemberKind::Izutsumi => Self::izutsumi(),
            PartyMemberKind::Shuro => Self::shuro(),
            PartyMemberKind::Namari => Self::namari(),
            PartyMemberKind::Kabru => Self::kabru(),
            PartyMemberKind::Rinsha => Self::rinsha(),
        }
    }
}

impl PartyMemberKind {
    /// キャラ別のレベルアップ時ステータス成長値
    pub fn stat_growth(self) -> StatGrowth {
        match self {
            PartyMemberKind::Laios => StatGrowth {
                hp: 5,
                mp: 1,
                attack: 2,
                defense: 1,
                speed: 1,
            },
            PartyMemberKind::Chilchuck => StatGrowth {
                hp: 3,
                mp: 0,
                attack: 2,
                defense: 1,
                speed: 2,
            },
            PartyMemberKind::Marcille => StatGrowth {
                hp: 3,
                mp: 3,
                attack: 1,
                defense: 1,
                speed: 1,
            },
            PartyMemberKind::Senshi => StatGrowth {
                hp: 6,
                mp: 0,
                attack: 2,
                defense: 2,
                speed: 0,
            },
            PartyMemberKind::Falin => StatGrowth {
                hp: 4,
                mp: 2,
                attack: 1,
                defense: 1,
                speed: 1,
            },
            PartyMemberKind::Izutsumi => StatGrowth {
                hp: 3,
                mp: 1,
                attack: 2,
                defense: 0,
                speed: 2,
            },
            PartyMemberKind::Shuro => StatGrowth {
                hp: 4,
                mp: 0,
                attack: 3,
                defense: 1,
                speed: 1,
            },
            PartyMemberKind::Namari => StatGrowth {
                hp: 5,
                mp: 0,
                attack: 2,
                defense: 2,
                speed: 0,
            },
            PartyMemberKind::Kabru => StatGrowth {
                hp: 4,
                mp: 1,
                attack: 2,
                defense: 1,
                speed: 1,
            },
            PartyMemberKind::Rinsha => StatGrowth {
                hp: 3,
                mp: 2,
                attack: 1,
                defense: 1,
                speed: 1,
            },
        }
    }
}

pub fn default_party() -> Vec<PartyMember> {
    vec![
        PartyMember::laios(),
        PartyMember::marcille(),
        PartyMember::falin(),
    ]
}

/// ゲーム開始時の初期パーティ（ライオスのみ）
pub fn initial_party() -> Vec<PartyMember> {
    vec![PartyMember::laios()]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecruitmentPath {
    /// 居酒屋2回で仲間になる
    TavernBond,
    /// 金を払って雇う
    GoldHire { cost: u32 },
}

impl PartyMemberKind {
    pub fn recruit_method(self) -> RecruitmentPath {
        match self {
            PartyMemberKind::Namari => RecruitmentPath::GoldHire { cost: 200 },
            PartyMemberKind::Chilchuck => RecruitmentPath::GoldHire { cost: 200 },
            _ => RecruitmentPath::TavernBond,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_to_next_level_values() {
        assert_eq!(exp_to_next_level(1), 10);  // Lv1→2
        assert_eq!(exp_to_next_level(2), 30);  // Lv2→3
        assert_eq!(exp_to_next_level(3), 60);  // Lv3→4
    }

    #[test]
    fn gain_exp_levels_up() {
        let mut laios = PartyMember::laios();
        let level_ups = laios.gain_exp(10); // ちょうどLv2に
        assert_eq!(level_ups, 1);
        assert_eq!(laios.level, 2);
        assert_eq!(laios.exp, 10);
        assert_eq!(laios.stats.max_hp, 35); // +5
        assert_eq!(laios.stats.attack, 10); // +2
    }

    #[test]
    fn gain_exp_multiple_level_ups() {
        let mut laios = PartyMember::laios();
        let level_ups = laios.gain_exp(60); // Lv1→2(10) → Lv2→3(30) → Lv3→4(60)
        assert_eq!(level_ups, 3);
        assert_eq!(laios.level, 4);
    }

    #[test]
    fn gain_exp_no_level_up() {
        let mut laios = PartyMember::laios();
        let level_ups = laios.gain_exp(5);
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
    fn effective_attack_without_weapon() {
        let laios = PartyMember::laios();
        assert_eq!(laios.effective_attack(), laios.stats.attack);
    }

    #[test]
    fn effective_attack_with_weapon() {
        use crate::equipment::WeaponKind;
        let mut laios = PartyMember::laios();
        laios.equipment.equip_weapon(WeaponKind::IronSword);
        assert_eq!(laios.effective_attack(), laios.stats.attack + 5);
    }
}
