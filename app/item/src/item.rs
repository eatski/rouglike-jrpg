use std::collections::HashMap;

use crate::equipment::{WeaponEntry, WeaponKind};

pub const INVENTORY_CAPACITY: u32 = 6;
pub const BAG_CAPACITY: u32 = 50;
pub const BAG_MEMBER_INDEX: usize = usize::MAX;

/// アイテム使用時の効果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemEffect {
    /// HP回復
    Heal { power: i32 },
    /// キーアイテム（説明表示のみ、消費しない）
    KeyItem,
    /// 素材（売却専用、使用不可）
    Material,
    /// 装備（武器を装備する）
    Equip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Herb,
    HighHerb,
    CopperKey,
    MoonFragment,
    MagicStone,
    SilverOre,
    AncientCoin,
    DragonScale,
    Weapon(WeaponKind),
}

impl ItemKind {
    pub fn name(self) -> &'static str {
        match self {
            ItemKind::Herb => "やくそう",
            ItemKind::HighHerb => "じょうやくそう",
            ItemKind::CopperKey => "どうのカギ",
            ItemKind::MoonFragment => "つきのかけら",
            ItemKind::MagicStone => "まほうのいし",
            ItemKind::SilverOre => "ぎんこうせき",
            ItemKind::AncientCoin => "いにしえのコイン",
            ItemKind::DragonScale => "りゅうのウロコ",
            ItemKind::Weapon(w) => w.name(),
        }
    }

    /// 武器バリアントの場合、WeaponKindを返す
    pub fn as_weapon(self) -> Option<WeaponKind> {
        match self {
            ItemKind::Weapon(w) => Some(w),
            _ => None,
        }
    }

}

pub static ALL_ITEMS: &[ItemKind] = &[
    ItemKind::Herb,
    ItemKind::HighHerb,
    ItemKind::CopperKey,
    ItemKind::MoonFragment,
    ItemKind::MagicStone,
    ItemKind::SilverOre,
    ItemKind::AncientCoin,
    ItemKind::DragonScale,
];

/// 全アイテムリストを返す
pub fn all_items() -> &'static [ItemKind] {
    ALL_ITEMS
}

/// アイテムパラメータエントリ
#[derive(Clone)]
pub struct ItemEntry {
    pub effect: ItemEffect,
    pub description: &'static str,
    pub price: u32,
    pub sell_price: u32,
}

/// アイテムパラメータテーブル
#[derive(Clone)]
pub struct ItemParamTable {
    item_entries: Vec<ItemEntry>,
    weapon_entries: Vec<WeaponEntry>,
    shop_items: Vec<ItemKind>,
    shop_weapons: Vec<WeaponKind>,
}

impl std::fmt::Debug for ItemParamTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemParamTable").finish()
    }
}

impl ItemParamTable {
    pub fn from_fn(
        item_fn: impl Fn(ItemKind) -> ItemEntry,
        weapon_fn: impl Fn(WeaponKind) -> WeaponEntry,
        shop_items: Vec<ItemKind>,
        shop_weapons: Vec<WeaponKind>,
    ) -> Self {
        let item_entries: Vec<ItemEntry> = ALL_ITEMS.iter().map(|&k| item_fn(k)).collect();
        let weapon_entries: Vec<WeaponEntry> =
            crate::equipment::ALL_WEAPONS.iter().map(|&k| weapon_fn(k)).collect();
        Self {
            item_entries,
            weapon_entries,
            shop_items,
            shop_weapons,
        }
    }

    fn item_index(item: ItemKind) -> usize {
        ALL_ITEMS.iter().position(|&k| k == item).expect("unknown ItemKind")
    }

    fn weapon_index(weapon: WeaponKind) -> usize {
        crate::equipment::ALL_WEAPONS
            .iter()
            .position(|&k| k == weapon)
            .expect("unknown WeaponKind")
    }

    // --- ItemKind アクセサ ---

    pub fn effect(&self, item: ItemKind) -> ItemEffect {
        match item {
            ItemKind::Weapon(_) => ItemEffect::Equip,
            _ => self.item_entries[Self::item_index(item)].effect,
        }
    }

    pub fn description(&self, item: ItemKind) -> &str {
        match item {
            ItemKind::Weapon(w) => self.weapon_entries[Self::weapon_index(w)].description,
            _ => self.item_entries[Self::item_index(item)].description,
        }
    }

    pub fn price(&self, item: ItemKind) -> u32 {
        match item {
            ItemKind::Weapon(w) => self.weapon_entries[Self::weapon_index(w)].price,
            _ => self.item_entries[Self::item_index(item)].price,
        }
    }

    pub fn sell_price(&self, item: ItemKind) -> u32 {
        match item {
            ItemKind::Weapon(w) => self.weapon_entries[Self::weapon_index(w)].price / 2,
            _ => self.item_entries[Self::item_index(item)].sell_price,
        }
    }

    pub fn is_consumable(&self, item: ItemKind) -> bool {
        matches!(self.effect(item), ItemEffect::Heal { .. })
    }

    // --- WeaponKind アクセサ ---

    pub fn weapon_attack_bonus(&self, weapon: WeaponKind) -> i32 {
        self.weapon_entries[Self::weapon_index(weapon)].attack_bonus
    }

    pub fn weapon_price(&self, weapon: WeaponKind) -> u32 {
        self.weapon_entries[Self::weapon_index(weapon)].price
    }

    pub fn weapon_description(&self, weapon: WeaponKind) -> &str {
        self.weapon_entries[Self::weapon_index(weapon)].description
    }

    // --- ショップデータ ---

    pub fn shop_items(&self) -> &[ItemKind] {
        &self.shop_items
    }

    pub fn shop_weapons(&self) -> &[WeaponKind] {
        &self.shop_weapons
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    items: HashMap<ItemKind, u32>,
    capacity: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            capacity: INVENTORY_CAPACITY,
        }
    }

    pub fn with_capacity(capacity: u32) -> Self {
        Self {
            items: HashMap::new(),
            capacity,
        }
    }

    pub fn add(&mut self, item: ItemKind, count: u32) {
        *self.items.entry(item).or_insert(0) += count;
    }

    /// 全アイテム合計個数
    pub fn total_count(&self) -> u32 {
        self.items.values().sum()
    }

    /// 指定個数を追加できるか（容量チェック）
    pub fn can_add(&self, count: u32) -> bool {
        self.total_count() + count <= self.capacity
    }

    /// 容量チェック付き追加。成功したらtrue
    pub fn try_add(&mut self, item: ItemKind, count: u32) -> bool {
        if !self.can_add(count) {
            return false;
        }
        self.add(item, count);
        true
    }

    /// アイテムを1つ使用。成功したらtrue、在庫なしならfalse
    pub fn use_item(&mut self, item: ItemKind) -> bool {
        if let Some(count) = self.items.get_mut(&item)
            && *count > 0
        {
            *count -= 1;
            return true;
        }
        false
    }

    pub fn count(&self, item: ItemKind) -> u32 {
        self.items.get(&item).copied().unwrap_or(0)
    }

    /// アイテムを1つ取り除く（売却用、consumableチェックなし）。成功したらtrue
    pub fn remove_item(&mut self, item: ItemKind) -> bool {
        if let Some(count) = self.items.get_mut(&item)
            && *count > 0
        {
            *count -= 1;
            return true;
        }
        false
    }

    /// 所持しているアイテム一覧（個数1以上）
    pub fn owned_items(&self) -> Vec<ItemKind> {
        self.items
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(&kind, _)| kind)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.items.values().all(|&count| count == 0)
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_add_and_count() {
        let mut inv = Inventory::new();
        assert_eq!(inv.count(ItemKind::Herb), 0);
        inv.add(ItemKind::Herb, 2);
        assert_eq!(inv.count(ItemKind::Herb), 2);
    }

    #[test]
    fn inventory_use_item() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Herb, 1);
        assert!(inv.use_item(ItemKind::Herb));
        assert_eq!(inv.count(ItemKind::Herb), 0);
        assert!(!inv.use_item(ItemKind::Herb));
    }

    #[test]
    fn inventory_owned_items() {
        let mut inv = Inventory::new();
        assert!(inv.owned_items().is_empty());
        inv.add(ItemKind::Herb, 2);
        assert_eq!(inv.owned_items(), vec![ItemKind::Herb]);
    }

    #[test]
    fn inventory_is_empty() {
        let mut inv = Inventory::new();
        assert!(inv.is_empty());
        inv.add(ItemKind::Herb, 1);
        assert!(!inv.is_empty());
        inv.use_item(ItemKind::Herb);
        assert!(inv.is_empty());
    }

    #[test]
    fn inventory_total_count() {
        let mut inv = Inventory::new();
        assert_eq!(inv.total_count(), 0);
        inv.add(ItemKind::Herb, 3);
        assert_eq!(inv.total_count(), 3);
    }

    #[test]
    fn inventory_can_add() {
        let mut inv = Inventory::new();
        assert!(inv.can_add(6));
        assert!(!inv.can_add(7));
        inv.add(ItemKind::Herb, 5);
        assert!(inv.can_add(1));
        assert!(!inv.can_add(2));
    }

    #[test]
    fn inventory_try_add() {
        let mut inv = Inventory::new();
        assert!(inv.try_add(ItemKind::Herb, 6));
        assert_eq!(inv.total_count(), 6);
        assert!(!inv.try_add(ItemKind::Herb, 1));
        assert_eq!(inv.total_count(), 6);
    }

    #[test]
    fn inventory_with_capacity() {
        let mut inv = Inventory::with_capacity(50);
        assert!(inv.can_add(50));
        assert!(!inv.can_add(51));
        inv.add(ItemKind::Herb, 49);
        assert!(inv.can_add(1));
        assert!(!inv.can_add(2));
    }
}
