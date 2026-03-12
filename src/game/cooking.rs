// 烹饪系统
// 注意：具体的烹饪逻辑已整合到 state.rs 中
// 此文件保留用于未来扩展（如果需要更复杂的烹饪系统）

use super::inventory::Inventory;
use super::types::{CropType, DishType, ItemType};

/// 检查是否可以制作指定料理
#[allow(dead_code)]
pub fn can_cook_dish(inventory: &Inventory, dish: DishType) -> bool {
    let ingredients = get_dish_ingredients(dish);
    for (item, count) in ingredients {
        if inventory.count(&item) < count {
            return false;
        }
    }
    true
}

/// 获取料理所需食材
#[allow(dead_code)]
pub fn get_dish_ingredients(dish: DishType) -> Vec<(ItemType, u32)> {
    match dish {
        // 基础料理：使用作物
        DishType::PotatoPancake => vec![(ItemType::Crop(CropType::Potato), 2)],
        DishType::MelonJuice => vec![(ItemType::Crop(CropType::Melon), 1)],
        DishType::PumpkinPie => vec![(ItemType::Crop(CropType::Pumpkin), 2)],
        DishType::YamRoast => vec![(ItemType::Crop(CropType::Yam), 2)],
        DishType::BlueberryMuffin => vec![(ItemType::Crop(CropType::Blueberry), 3)],
        DishType::PepperSteak => vec![
            (ItemType::Crop(CropType::HotPepper), 1),
            (ItemType::Crop(CropType::Potato), 1),
        ],
        DishType::CranberrySauce => vec![(ItemType::Crop(CropType::Cranberry), 2)],
        DishType::ParsnipSoup => vec![(ItemType::Crop(CropType::Parsnip), 2)],
        DishType::GreenBeanSoup => vec![(ItemType::Crop(CropType::GreenBean), 2)],
        // 全家桶早餐：需要土豆煎饼（春之平原）+ 甜瓜汁（夏之海岛）
        // 强制玩家跨气候带交换！
        DishType::CompleteBreakfast => vec![
            (ItemType::Dish(DishType::PotatoPancake), 1),
            (ItemType::Dish(DishType::MelonJuice), 1),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_cook_dish() {
        let mut inventory = Inventory::new();

        // 没有材料，不能制作土豆煎饼
        assert!(!can_cook_dish(&inventory, DishType::PotatoPancake));

        // 添加2个土豆
        inventory.add(ItemType::Crop(CropType::Potato), 2);
        assert!(can_cook_dish(&inventory, DishType::PotatoPancake));
    }

    #[test]
    fn test_get_dish_ingredients() {
        let ingredients = get_dish_ingredients(DishType::CompleteBreakfast);
        assert_eq!(ingredients.len(), 2);
    }
}
