/// 装備スロット
#[derive(Debug, Clone)]
pub struct Equipment<K: Copy> {
    pub weapon: Option<K>,
}

impl<K: Copy> Default for Equipment<K> {
    fn default() -> Self {
        Self { weapon: None }
    }
}

impl<K: Copy> Equipment<K> {
    pub fn new() -> Self {
        Self { weapon: None }
    }

    /// 武器を装備し、以前の武器を返す
    pub fn equip_weapon(&mut self, weapon: K) -> Option<K> {
        self.weapon.replace(weapon)
    }
}

impl<K: crate::ItemLookup> Equipment<K> {
    /// 装備による攻撃力ボーナス合計
    pub fn attack_bonus(&self) -> i32 {
        self.weapon.map_or(0, |w| w.entry().attack_bonus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{ItemEffect, ItemEntry, ItemLookup};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestWeapon {
        Sword,
        Staff,
    }

    impl ItemLookup for TestWeapon {
        fn entry(&self) -> ItemEntry<Self> {
            match self {
                TestWeapon::Sword => ItemEntry {
                    key: TestWeapon::Sword,
                    name: "sword",
                    effect: ItemEffect::Material,
                    description: "",
                    price: 10,
                    sell_price: 5,
                    attack_bonus: 5,
                },
                TestWeapon::Staff => ItemEntry {
                    key: TestWeapon::Staff,
                    name: "staff",
                    effect: ItemEffect::Material,
                    description: "",
                    price: 30,
                    sell_price: 15,
                    attack_bonus: 3,
                },
            }
        }
    }

    #[test]
    fn equip_weapon_returns_previous() {
        let mut eq = Equipment::new();
        let prev = eq.equip_weapon(TestWeapon::Sword);
        assert_eq!(prev, None);
        assert_eq!(eq.weapon, Some(TestWeapon::Sword));
        assert_eq!(eq.attack_bonus(), 5);

        let prev = eq.equip_weapon(TestWeapon::Staff);
        assert_eq!(prev, Some(TestWeapon::Sword));
        assert_eq!(eq.attack_bonus(), 3);
    }
}
