// 背包系统

use super::types::{CropType, DishType, ItemType};
use serde::{Deserialize, Serialize};

/// 背包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// 物品列表 (物品, 数量)
    items: Vec<(ItemType, u32)>,
    /// 金币
    pub gold: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            gold: 500, // 初始金币
        }
    }

    /// 查找物品索引
    fn find_item(&self, item_type: &ItemType) -> Option<usize> {
        self.items.iter().position(|(t, _)| t == item_type)
    }

    /// 添加物品
    pub fn add(&mut self, item_type: ItemType, count: u32) {
        if let Some(idx) = self.find_item(&item_type) {
            self.items[idx].1 += count;
        } else {
            self.items.push((item_type, count));
        }
    }

    /// 使用物品（单个）
    pub fn use_item(&mut self, item_type: &ItemType) -> bool {
        self.use_items(item_type, 1)
    }

    /// 使用物品（指定数量）
    pub fn use_items(&mut self, item_type: &ItemType, count: u32) -> bool {
        if let Some(idx) = self.find_item(item_type) {
            if self.items[idx].1 >= count {
                self.items[idx].1 -= count;
                if self.items[idx].1 == 0 {
                    self.items.remove(idx);
                }
                return true;
            }
        }
        false
    }

    /// 使用种子
    pub fn use_seed(&mut self, crop: CropType) -> bool {
        self.use_item(&ItemType::Seed(crop))
    }

    /// 使用作物
    pub fn use_crop(&mut self, crop: CropType, count: u32) -> bool {
        self.use_items(&ItemType::Crop(crop), count)
    }

    /// 检查是否有足够的物品
    pub fn has_item(&self, item_type: &ItemType, count: u32) -> bool {
        self.count(item_type) >= count
    }

    /// 检查是否有足够的作物
    pub fn has_crop(&self, crop: CropType, count: u32) -> bool {
        self.has_item(&ItemType::Crop(crop), count)
    }

    /// 获取物品数量
    pub fn count(&self, item_type: &ItemType) -> u32 {
        self.find_item(item_type)
            .map(|idx| self.items[idx].1)
            .unwrap_or(0)
    }

    /// 花费金币
    pub fn spend_gold(&mut self, amount: u32) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    /// 获得金币
    pub fn earn_gold(&mut self, amount: u32) {
        self.gold += amount;
    }

    /// 购买种子
    pub fn buy_seed(&mut self, crop: CropType, count: u32) -> bool {
        let price = crop.seed_price() * count;
        if self.spend_gold(price) {
            self.add(ItemType::Seed(crop), count);
            true
        } else {
            false
        }
    }

    /// 出售作物
    pub fn sell_crop(&mut self, crop: CropType, count: u32) -> bool {
        let item = ItemType::Crop(crop);
        if self.use_items(&item, count) {
            self.earn_gold(crop.sell_price() * count);
            true
        } else {
            false
        }
    }

    /// 出售物品
    pub fn sell_item(&mut self, item_type: &ItemType, count: u32) -> bool {
        if self.use_items(item_type, count) {
            self.earn_gold(item_type.sell_price() * count);
            true
        } else {
            false
        }
    }

    /// 列出所有作物
    pub fn list_crops(&self) -> Vec<(CropType, u32)> {
        let mut crops: Vec<(CropType, u32)> = self
            .items
            .iter()
            .filter_map(|(item, count)| {
                if let ItemType::Crop(crop) = item {
                    Some((*crop, *count))
                } else {
                    None
                }
            })
            .collect();
        crops.sort_by_key(|(c, _)| c.name());
        crops
    }

    /// 列出所有种子
    pub fn list_seeds(&self) -> Vec<(CropType, u32)> {
        let mut seeds: Vec<(CropType, u32)> = self
            .items
            .iter()
            .filter_map(|(item, count)| {
                if let ItemType::Seed(crop) = item {
                    Some((*crop, *count))
                } else {
                    None
                }
            })
            .collect();
        seeds.sort_by_key(|(c, _)| c.name());
        seeds
    }

    /// 列出所有果酱
    pub fn list_jellies(&self) -> Vec<(CropType, u32)> {
        let mut jellies: Vec<(CropType, u32)> = self
            .items
            .iter()
            .filter_map(|(item, count)| {
                if let ItemType::Jelly(crop) = item {
                    Some((*crop, *count))
                } else {
                    None
                }
            })
            .collect();
        jellies.sort_by_key(|(c, _)| c.name());
        jellies
    }

    /// 列出所有料理
    pub fn list_dishes(&self) -> Vec<(DishType, u32)> {
        let mut dishes: Vec<(DishType, u32)> = self
            .items
            .iter()
            .filter_map(|(item, count)| {
                if let ItemType::Dish(dish) = item {
                    Some((*dish, *count))
                } else {
                    None
                }
            })
            .collect();
        dishes.sort_by_key(|(d, _)| d.name());
        dishes
    }

    /// 获取物品总数
    pub fn total_items(&self) -> u32 {
        self.items.iter().map(|(_, c)| *c).sum()
    }

    /// 获取物品列表（用于显示）
    pub fn list_all(&self) -> Vec<(ItemType, u32)> {
        let mut items = self.items.clone();
        items.sort_by(|a, b| {
            match (&a.0, &b.0) {
                (ItemType::Crop(c1), ItemType::Crop(c2)) => c1.name().cmp(c2.name()),
                (ItemType::Seed(c1), ItemType::Seed(c2)) => c1.name().cmp(c2.name()),
                (ItemType::Jelly(c1), ItemType::Jelly(c2)) => c1.name().cmp(c2.name()),
                (ItemType::Dish(d1), ItemType::Dish(d2)) => d1.name().cmp(d2.name()),
                // 排序优先级：作物 < 种子 < 果酱 < 料理
                (ItemType::Crop(_), _) => std::cmp::Ordering::Less,
                (ItemType::Seed(_), ItemType::Crop(_)) => std::cmp::Ordering::Greater,
                (ItemType::Seed(_), _) => std::cmp::Ordering::Less,
                (ItemType::Jelly(_), ItemType::Dish(_)) => std::cmp::Ordering::Less,
                (ItemType::Jelly(_), _) => std::cmp::Ordering::Greater,
                (ItemType::Dish(_), _) => std::cmp::Ordering::Greater,
            }
        });
        items
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
