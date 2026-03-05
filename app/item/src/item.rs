use std::collections::HashMap;
use std::hash::Hash;

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
}

/// アイテムデータからエントリを取得するトレイト
pub trait ItemLookup: Copy + Eq + Hash {
    fn entry(&self) -> ItemEntry<Self>;
}

/// アイテムパラメータエントリ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemEntry<K> {
    pub key: K,
    pub name: &'static str,
    pub effect: ItemEffect,
    pub description: &'static str,
    pub price: u32,
    pub sell_price: u32,
    pub attack_bonus: i32,
}

impl<K: Copy> ItemEntry<K> {
    pub const fn as_key(&self) -> K {
        self.key
    }
    pub const fn is_weapon(&self) -> bool {
        self.attack_bonus > 0
    }
    pub const fn is_consumable(&self) -> bool {
        matches!(self.effect, ItemEffect::Heal { .. })
    }
}

#[derive(Debug, Clone)]
pub struct Inventory<K: Eq + Hash> {
    items: HashMap<K, u32>,
    capacity: u32,
}

impl<K: Copy + Eq + Hash> Inventory<K> {
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

    pub fn add(&mut self, item: K, count: u32) {
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
    pub fn try_add(&mut self, item: K, count: u32) -> bool {
        if !self.can_add(count) {
            return false;
        }
        self.add(item, count);
        true
    }

    /// アイテムを1つ使用。成功したらtrue、在庫なしならfalse
    pub fn use_item(&mut self, item: K) -> bool {
        if let Some(count) = self.items.get_mut(&item)
            && *count > 0
        {
            *count -= 1;
            return true;
        }
        false
    }

    pub fn count(&self, item: K) -> u32 {
        self.items.get(&item).copied().unwrap_or(0)
    }

    /// アイテムを1つ取り除く（売却用、consumableチェックなし）。成功したらtrue
    pub fn remove_item(&mut self, item: K) -> bool {
        if let Some(count) = self.items.get_mut(&item)
            && *count > 0
        {
            *count -= 1;
            return true;
        }
        false
    }

    /// 所持しているアイテム一覧（個数1以上）
    pub fn owned_items(&self) -> Vec<K> {
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

impl<K: Copy + Eq + Hash> Default for Inventory<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestItem {
        A,
        B,
    }

    #[test]
    fn inventory_add_and_count() {
        let mut inv = Inventory::new();
        assert_eq!(inv.count(TestItem::A), 0);
        inv.add(TestItem::A, 2);
        assert_eq!(inv.count(TestItem::A), 2);
    }

    #[test]
    fn inventory_use_item() {
        let mut inv = Inventory::new();
        inv.add(TestItem::A, 1);
        assert!(inv.use_item(TestItem::A));
        assert_eq!(inv.count(TestItem::A), 0);
        assert!(!inv.use_item(TestItem::A));
    }

    #[test]
    fn inventory_owned_items() {
        let mut inv = Inventory::new();
        assert!(inv.owned_items().is_empty());
        inv.add(TestItem::A, 2);
        assert_eq!(inv.owned_items(), vec![TestItem::A]);
    }

    #[test]
    fn inventory_is_empty() {
        let mut inv = Inventory::new();
        assert!(inv.is_empty());
        inv.add(TestItem::A, 1);
        assert!(!inv.is_empty());
        inv.use_item(TestItem::A);
        assert!(inv.is_empty());
    }

    #[test]
    fn inventory_total_count() {
        let mut inv = Inventory::new();
        assert_eq!(inv.total_count(), 0);
        inv.add(TestItem::A, 3);
        assert_eq!(inv.total_count(), 3);
    }

    #[test]
    fn inventory_can_add() {
        let mut inv = Inventory::new();
        assert!(inv.can_add(6));
        assert!(!inv.can_add(7));
        inv.add(TestItem::A, 5);
        assert!(inv.can_add(1));
        assert!(!inv.can_add(2));
    }

    #[test]
    fn inventory_try_add() {
        let mut inv = Inventory::new();
        assert!(inv.try_add(TestItem::A, 6));
        assert_eq!(inv.total_count(), 6);
        assert!(!inv.try_add(TestItem::A, 1));
        assert_eq!(inv.total_count(), 6);
    }

    #[test]
    fn inventory_with_capacity() {
        let mut inv = Inventory::with_capacity(50);
        assert!(inv.can_add(50));
        assert!(!inv.can_add(51));
        inv.add(TestItem::A, 49);
        assert!(inv.can_add(1));
        assert!(!inv.can_add(2));
    }
}
