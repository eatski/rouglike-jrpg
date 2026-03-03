use item::{
    ItemEffect::*, ItemEntry, ItemKind, ItemParamTable, WeaponEntry, WeaponKind,
};

pub fn item_param_table() -> ItemParamTable {
    ItemParamTable::from_fn(
        |item| match item {
            ItemKind::Herb => ItemEntry {
                effect: Heal { power: 25 },
                description: "HPを かいふくする やくそう",
                price: 8,
                sell_price: 4,
            },
            ItemKind::HighHerb => ItemEntry {
                effect: Heal { power: 50 },
                description: "HPを おおきく かいふくする",
                price: 24,
                sell_price: 12,
            },
            ItemKind::CopperKey => ItemEntry {
                effect: KeyItem,
                description: "どこかの とびらを あけるカギ",
                price: 0,
                sell_price: 0,
            },
            ItemKind::MoonFragment => ItemEntry {
                effect: Material,
                description: "ほこらの とびらを ひらく ふしぎな かけら",
                price: 50,
                sell_price: 25,
            },
            ItemKind::MagicStone => ItemEntry {
                effect: Material,
                description: "ふしぎな ちからを もつ いし",
                price: 0,
                sell_price: 30,
            },
            ItemKind::SilverOre => ItemEntry {
                effect: Material,
                description: "きれいな ぎんいろの こうせき",
                price: 0,
                sell_price: 60,
            },
            ItemKind::AncientCoin => ItemEntry {
                effect: Material,
                description: "おおむかしの きんか",
                price: 0,
                sell_price: 120,
            },
            ItemKind::DragonScale => ItemEntry {
                effect: Material,
                description: "りゅうの からだを おおう ウロコ",
                price: 0,
                sell_price: 250,
            },
            ItemKind::Weapon(_) => unreachable!("Weapon variants are handled by weapon_fn"),
        },
        |weapon| match weapon {
            WeaponKind::WoodenSword => WeaponEntry {
                attack_bonus: 2,
                price: 10,
                description: "きで つくった つるぎ",
            },
            WeaponKind::IronSword => WeaponEntry {
                attack_bonus: 5,
                price: 50,
                description: "てつで きたえた つるぎ",
            },
            WeaponKind::SteelSword => WeaponEntry {
                attack_bonus: 10,
                price: 150,
                description: "はがねの かたい つるぎ",
            },
            WeaponKind::MageStaff => WeaponEntry {
                attack_bonus: 3,
                price: 30,
                description: "まりょくを たかめる つえ",
            },
            WeaponKind::HolyStaff => WeaponEntry {
                attack_bonus: 4,
                price: 80,
                description: "せいなる ちからの つえ",
            },
        },
        vec![ItemKind::Herb, ItemKind::HighHerb, ItemKind::MoonFragment],
        vec![
            WeaponKind::WoodenSword,
            WeaponKind::IronSword,
            WeaponKind::MageStaff,
        ],
    )
}
