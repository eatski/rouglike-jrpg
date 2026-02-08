use crate::battle::PartyMember;

/// パーティ全員のHP/MPを全回復する
pub fn heal_party(party: &mut [PartyMember]) {
    for member in party.iter_mut() {
        member.stats.hp = member.stats.max_hp;
        member.stats.mp = member.stats.max_mp;
    }
}

/// NPCの台詞を返す
pub fn townsperson_dialogue() -> &'static str {
    "このさきに まおうの しろが あるらしいぞ"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::default_party;

    #[test]
    fn heal_party_restores_full_hp_mp() {
        let mut party = default_party();
        // ダメージを与える
        party[0].stats.hp = 1;
        party[0].stats.mp = 0;
        party[1].stats.hp = 5;

        heal_party(&mut party);

        for member in &party {
            assert_eq!(member.stats.hp, member.stats.max_hp);
            assert_eq!(member.stats.mp, member.stats.max_mp);
        }
    }

    #[test]
    fn townsperson_dialogue_returns_non_empty() {
        let dialogue = townsperson_dialogue();
        assert!(!dialogue.is_empty());
    }
}
