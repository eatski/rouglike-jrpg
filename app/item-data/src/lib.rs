use item::{ItemEffect, ItemEntry, ItemLookup};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKey {
    Herb,
    HighHerb,
    CopperKey,
    MoonFragment,
    MagicStone,
    SilverOre,
    AncientCoin,
    DragonScale,
    WoodenSword,
    IronSword,
    SteelSword,
    MageStaff,
    HolyStaff,
}

pub const HERB: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::Herb,
    name: "やくそう",
    effect: ItemEffect::Heal { power: 25 },
    description: "HPを かいふくする やくそう",
    price: 8,
    sell_price: 4,
    attack_bonus: 0,
};

pub const HIGH_HERB: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::HighHerb,
    name: "じょうやくそう",
    effect: ItemEffect::Heal { power: 50 },
    description: "HPを おおきく かいふくする",
    price: 24,
    sell_price: 12,
    attack_bonus: 0,
};

pub const COPPER_KEY: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::CopperKey,
    name: "どうのカギ",
    effect: ItemEffect::KeyItem,
    description: "どこかの とびらを あけるカギ",
    price: 0,
    sell_price: 0,
    attack_bonus: 0,
};

pub const MOON_FRAGMENT: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::MoonFragment,
    name: "つきのかけら",
    effect: ItemEffect::Material,
    description: "ほこらの とびらを ひらく ふしぎな かけら",
    price: 50,
    sell_price: 25,
    attack_bonus: 0,
};

pub const MAGIC_STONE: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::MagicStone,
    name: "まほうのいし",
    effect: ItemEffect::Material,
    description: "ふしぎな ちからを もつ いし",
    price: 0,
    sell_price: 30,
    attack_bonus: 0,
};

pub const SILVER_ORE: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::SilverOre,
    name: "ぎんこうせき",
    effect: ItemEffect::Material,
    description: "きれいな ぎんいろの こうせき",
    price: 0,
    sell_price: 60,
    attack_bonus: 0,
};

pub const ANCIENT_COIN: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::AncientCoin,
    name: "いにしえのコイン",
    effect: ItemEffect::Material,
    description: "おおむかしの きんか",
    price: 0,
    sell_price: 120,
    attack_bonus: 0,
};

pub const DRAGON_SCALE: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::DragonScale,
    name: "りゅうのウロコ",
    effect: ItemEffect::Material,
    description: "りゅうの からだを おおう ウロコ",
    price: 0,
    sell_price: 250,
    attack_bonus: 0,
};

pub const WOODEN_SWORD: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::WoodenSword,
    name: "きのつるぎ",
    effect: ItemEffect::Material,
    description: "きで つくった つるぎ",
    price: 10,
    sell_price: 5,
    attack_bonus: 2,
};

pub const IRON_SWORD: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::IronSword,
    name: "てつのつるぎ",
    effect: ItemEffect::Material,
    description: "てつで きたえた つるぎ",
    price: 50,
    sell_price: 25,
    attack_bonus: 5,
};

pub const STEEL_SWORD: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::SteelSword,
    name: "はがねのつるぎ",
    effect: ItemEffect::Material,
    description: "はがねの かたい つるぎ",
    price: 150,
    sell_price: 75,
    attack_bonus: 10,
};

pub const MAGE_STAFF: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::MageStaff,
    name: "まどうしのつえ",
    effect: ItemEffect::Material,
    description: "まりょくを たかめる つえ",
    price: 30,
    sell_price: 15,
    attack_bonus: 3,
};

pub const HOLY_STAFF: ItemEntry<ItemKey> = ItemEntry {
    key: ItemKey::HolyStaff,
    name: "せいなるつえ",
    effect: ItemEffect::Material,
    description: "せいなる ちからの つえ",
    price: 80,
    sell_price: 40,
    attack_bonus: 4,
};

pub static ALL_ITEMS: &[ItemEntry<ItemKey>] = &[
    HERB,
    HIGH_HERB,
    COPPER_KEY,
    MOON_FRAGMENT,
    MAGIC_STONE,
    SILVER_ORE,
    ANCIENT_COIN,
    DRAGON_SCALE,
    WOODEN_SWORD,
    IRON_SWORD,
    STEEL_SWORD,
    MAGE_STAFF,
    HOLY_STAFF,
];

pub static ALL_ITEM_KEYS: &[ItemKey] = &[
    ItemKey::Herb,
    ItemKey::HighHerb,
    ItemKey::CopperKey,
    ItemKey::MoonFragment,
    ItemKey::MagicStone,
    ItemKey::SilverOre,
    ItemKey::AncientCoin,
    ItemKey::DragonScale,
    ItemKey::WoodenSword,
    ItemKey::IronSword,
    ItemKey::SteelSword,
    ItemKey::MageStaff,
    ItemKey::HolyStaff,
];

pub static SHOP_ITEMS: &[ItemKey] = &[
    ItemKey::Herb,
    ItemKey::HighHerb,
    ItemKey::MoonFragment,
];

pub static SHOP_WEAPONS: &[ItemKey] = &[
    ItemKey::WoodenSword,
    ItemKey::IronSword,
    ItemKey::MageStaff,
];

impl ItemKey {
    pub const fn entry(self) -> ItemEntry<ItemKey> {
        match self {
            ItemKey::Herb => HERB,
            ItemKey::HighHerb => HIGH_HERB,
            ItemKey::CopperKey => COPPER_KEY,
            ItemKey::MoonFragment => MOON_FRAGMENT,
            ItemKey::MagicStone => MAGIC_STONE,
            ItemKey::SilverOre => SILVER_ORE,
            ItemKey::AncientCoin => ANCIENT_COIN,
            ItemKey::DragonScale => DRAGON_SCALE,
            ItemKey::WoodenSword => WOODEN_SWORD,
            ItemKey::IronSword => IRON_SWORD,
            ItemKey::SteelSword => STEEL_SWORD,
            ItemKey::MageStaff => MAGE_STAFF,
            ItemKey::HolyStaff => HOLY_STAFF,
        }
    }

    pub const fn name(self) -> &'static str {
        self.entry().name
    }

    pub const fn is_weapon(self) -> bool {
        self.entry().attack_bonus > 0
    }
}

impl ItemLookup for ItemKey {
    fn entry(&self) -> ItemEntry<ItemKey> {
        (*self).entry()
    }
}
