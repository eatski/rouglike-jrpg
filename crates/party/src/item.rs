use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Herb,
}

impl ItemKind {
    pub fn name(self) -> &'static str {
        match self {
            ItemKind::Herb => "やくそう",
        }
    }

    pub fn heal_power(self) -> i32 {
        match self {
            ItemKind::Herb => 25,
        }
    }

    pub fn price(self) -> u32 {
        match self {
            ItemKind::Herb => 8,
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
}
