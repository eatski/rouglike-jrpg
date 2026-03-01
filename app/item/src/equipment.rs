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

    pub fn description(self) -> &'static str {
        match self {
            WeaponKind::WoodenSword => "きで つくった つるぎ",
            WeaponKind::IronSword => "てつで きたえた つるぎ",
            WeaponKind::SteelSword => "はがねの かたい つるぎ",
            WeaponKind::MageStaff => "まりょくを たかめる つえ",
            WeaponKind::HolyStaff => "せいなる ちからの つえ",
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
}
