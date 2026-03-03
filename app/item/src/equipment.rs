/// 武器の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponKind {
    WoodenSword,
    IronSword,
    SteelSword,
    MageStaff,
    HolyStaff,
}

impl WeaponKind {
    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::WoodenSword => "きのつるぎ",
            WeaponKind::IronSword => "てつのつるぎ",
            WeaponKind::SteelSword => "はがねのつるぎ",
            WeaponKind::MageStaff => "まどうしのつえ",
            WeaponKind::HolyStaff => "せいなるつえ",
        }
    }

}

/// 武器パラメータエントリ
#[derive(Clone)]
pub struct WeaponEntry {
    pub attack_bonus: i32,
    pub price: u32,
    pub description: &'static str,
}

pub static ALL_WEAPONS: &[WeaponKind] = &[
    WeaponKind::WoodenSword,
    WeaponKind::IronSword,
    WeaponKind::SteelSword,
    WeaponKind::MageStaff,
    WeaponKind::HolyStaff,
];

pub fn all_weapons() -> &'static [WeaponKind] {
    ALL_WEAPONS
}

/// 装備スロット
#[derive(Debug, Clone, Default)]
pub struct Equipment {
    pub weapon: Option<WeaponKind>,
}

impl Equipment {
    pub fn new() -> Self {
        Self { weapon: None }
    }

    /// 武器を装備し、以前の武器を返す
    pub fn equip_weapon(&mut self, weapon: WeaponKind) -> Option<WeaponKind> {
        self.weapon.replace(weapon)
    }

    /// 装備による攻撃力ボーナス合計
    pub fn attack_bonus(&self, table: &crate::item::ItemParamTable) -> i32 {
        self.weapon.map_or(0, |w| table.weapon_attack_bonus(w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_table() -> crate::item::ItemParamTable {
        use crate::item::{ItemEffect, ItemEntry, ItemKind};
        crate::item::ItemParamTable::from_fn(
            |item| ItemEntry {
                effect: match item {
                    ItemKind::Herb => ItemEffect::Heal { power: 25 },
                    _ => ItemEffect::Material,
                },
                description: "",
                price: 0,
                sell_price: 0,
            },
            |weapon| WeaponEntry {
                attack_bonus: match weapon {
                    WeaponKind::WoodenSword => 2,
                    WeaponKind::IronSword => 5,
                    WeaponKind::SteelSword => 10,
                    WeaponKind::MageStaff => 3,
                    WeaponKind::HolyStaff => 4,
                },
                price: 0,
                description: "",
            },
            vec![],
            vec![],
        )
    }

    #[test]
    fn equip_weapon_returns_previous() {
        let table = test_table();
        let mut eq = Equipment::new();
        let prev = eq.equip_weapon(WeaponKind::WoodenSword);
        assert_eq!(prev, None);
        assert_eq!(eq.weapon, Some(WeaponKind::WoodenSword));
        assert_eq!(eq.attack_bonus(&table), 2);

        let prev = eq.equip_weapon(WeaponKind::IronSword);
        assert_eq!(prev, Some(WeaponKind::WoodenSword));
        assert_eq!(eq.attack_bonus(&table), 5);
    }
}
