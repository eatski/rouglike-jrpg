use crate::equipment::Equipment;
use crate::item::Inventory;
use crate::stats::CombatStats;

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
    pub inventory: Inventory,
    pub equipment: Equipment,
}

impl PartyMember {
    /// 装備込みの実効攻撃力
    pub fn effective_attack(&self) -> i32 {
        self.stats.attack + self.equipment.attack_bonus()
    }

    pub fn hero() -> Self {
        Self {
            kind: PartyMemberKind::Hero,
            stats: CombatStats::new(30, 8, 3, 5, 5),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn mage() -> Self {
        Self {
            kind: PartyMemberKind::Mage,
            stats: CombatStats::new(20, 10, 2, 7, 15),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn priest() -> Self {
        Self {
            kind: PartyMemberKind::Priest,
            stats: CombatStats::new(25, 5, 4, 4, 12),
            inventory: Inventory::new(),
            equipment: Equipment::new(),
        }
    }

    pub fn from_kind(kind: PartyMemberKind) -> Self {
        match kind {
            PartyMemberKind::Hero => Self::hero(),
            PartyMemberKind::Mage => Self::mage(),
            PartyMemberKind::Priest => Self::priest(),
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

/// ゲーム開始時の初期パーティ（勇者のみ）
pub fn initial_party() -> Vec<PartyMember> {
    vec![PartyMember::hero()]
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

/// 全仲間候補リストを返す（勇者以外）
pub fn default_candidates() -> Vec<RecruitCandidate> {
    vec![
        RecruitCandidate::new(PartyMemberKind::Mage),
        RecruitCandidate::new(PartyMemberKind::Priest),
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

    #[test]
    fn initial_party_has_one_member() {
        let party = initial_party();
        assert_eq!(party.len(), 1);
        assert_eq!(party[0].kind, PartyMemberKind::Hero);
    }

    #[test]
    fn default_candidates_are_mage_and_priest() {
        let candidates = default_candidates();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].kind, PartyMemberKind::Mage);
        assert_eq!(candidates[1].kind, PartyMemberKind::Priest);
        assert_eq!(candidates[0].status, RecruitmentStatus::Undiscovered);
        assert_eq!(candidates[1].status, RecruitmentStatus::Undiscovered);
    }

    #[test]
    fn talk_to_candidate_transitions() {
        let mut candidate = RecruitCandidate::new(PartyMemberKind::Mage);

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
    fn from_kind_creates_correct_member() {
        let hero = PartyMember::from_kind(PartyMemberKind::Hero);
        assert_eq!(hero.kind, PartyMemberKind::Hero);
        assert_eq!(hero.stats.max_hp, 30);

        let mage = PartyMember::from_kind(PartyMemberKind::Mage);
        assert_eq!(mage.kind, PartyMemberKind::Mage);
        assert_eq!(mage.stats.max_hp, 20);
    }

    #[test]
    fn effective_attack_without_weapon() {
        let hero = PartyMember::hero();
        assert_eq!(hero.effective_attack(), hero.stats.attack);
    }

    #[test]
    fn effective_attack_with_weapon() {
        use crate::equipment::WeaponKind;
        let mut hero = PartyMember::hero();
        hero.equipment.equip_weapon(WeaponKind::IronSword);
        assert_eq!(hero.effective_attack(), hero.stats.attack + 5);
    }
}
