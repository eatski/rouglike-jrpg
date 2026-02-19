use std::collections::HashMap;

pub const INVENTORY_CAPACITY: u32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Herb,
    CopperKey,
}

impl ItemKind {
    pub fn name(self) -> &'static str {
        match self {
            ItemKind::Herb => "やくそう",
            ItemKind::CopperKey => "どうのカギ",
        }
    }

    pub fn heal_power(self) -> i32 {
        match self {
            ItemKind::Herb => 25,
            ItemKind::CopperKey => 0,
        }
    }

    pub fn price(self) -> u32 {
        match self {
            ItemKind::Herb => 8,
            ItemKind::CopperKey => 0,
        }
    }
}

/// 道具屋で購入可能なアイテム一覧
pub fn shop_items() -> Vec<ItemKind> {
    vec![ItemKind::Herb]
}

/// 全アイテムリストを返す
pub fn all_items() -> Vec<ItemKind> {
    vec![ItemKind::Herb]
}

#[derive(Debug, Clone)]
pub struct Inventory {
    items: HashMap<ItemKind, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
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
        self.total_count() + count <= INVENTORY_CAPACITY
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
    fn herb_properties() {
        assert_eq!(ItemKind::Herb.name(), "やくそう");
        assert_eq!(ItemKind::Herb.heal_power(), 25);
    }

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
    fn all_items_returns_herb() {
        let items = all_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], ItemKind::Herb);
    }

    #[test]
    fn herb_price() {
        assert_eq!(ItemKind::Herb.price(), 8);
    }

    #[test]
    fn shop_items_returns_herb() {
        let items = shop_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], ItemKind::Herb);
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
}
