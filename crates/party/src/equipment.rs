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

    pub fn attack_bonus(self) -> i32 {
        match self {
            WeaponKind::WoodenSword => 2,
            WeaponKind::IronSword => 5,
            WeaponKind::SteelSword => 10,
            WeaponKind::MageStaff => 3,
            WeaponKind::HolyStaff => 4,
        }
    }

    pub fn price(self) -> u32 {
        match self {
            WeaponKind::WoodenSword => 10,
            WeaponKind::IronSword => 50,
            WeaponKind::SteelSword => 150,
            WeaponKind::MageStaff => 30,
            WeaponKind::HolyStaff => 80,
        }
    }
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
    pub fn attack_bonus(&self) -> i32 {
        self.weapon.map_or(0, |w| w.attack_bonus())
    }
}

/// 道具屋で購入可能な武器一覧
pub fn shop_weapons() -> Vec<WeaponKind> {
    vec![
        WeaponKind::WoodenSword,
        WeaponKind::IronSword,
        WeaponKind::MageStaff,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_properties() {
        assert_eq!(WeaponKind::WoodenSword.name(), "きのつるぎ");
        assert_eq!(WeaponKind::WoodenSword.attack_bonus(), 2);
        assert_eq!(WeaponKind::WoodenSword.price(), 10);
    }

    #[test]
    fn equipment_default_is_empty() {
        let eq = Equipment::new();
        assert_eq!(eq.weapon, None);
        assert_eq!(eq.attack_bonus(), 0);
    }

    #[test]
    fn equip_weapon_returns_previous() {
        let mut eq = Equipment::new();
        let prev = eq.equip_weapon(WeaponKind::WoodenSword);
        assert_eq!(prev, None);
        assert_eq!(eq.weapon, Some(WeaponKind::WoodenSword));
        assert_eq!(eq.attack_bonus(), 2);

        let prev = eq.equip_weapon(WeaponKind::IronSword);
        assert_eq!(prev, Some(WeaponKind::WoodenSword));
        assert_eq!(eq.attack_bonus(), 5);
    }

    #[test]
    fn shop_weapons_list() {
        let weapons = shop_weapons();
        assert_eq!(weapons.len(), 3);
        assert!(weapons.contains(&WeaponKind::WoodenSword));
        assert!(weapons.contains(&WeaponKind::IronSword));
        assert!(weapons.contains(&WeaponKind::MageStaff));
    }
}
