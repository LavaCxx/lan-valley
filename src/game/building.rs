// 建筑系统

use super::grid::Grid;
use super::inventory::Inventory;
use super::types::{Building, BuildingType, CropType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 建筑管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingManager {
    /// 已放置的建筑
    buildings: Vec<Building>,
    /// 已购买但未放置的建筑数量
    owned: HashMap<BuildingType, u32>,
}

impl BuildingManager {
    pub fn new() -> Self {
        Self {
            buildings: Vec::new(),
            owned: HashMap::new(),
        }
    }

    /// 购买建筑
    pub fn buy_building(&mut self, building_type: BuildingType) {
        *self.owned.entry(building_type).or_insert(0) += 1;
    }

    /// 使用已拥有的建筑
    pub fn use_owned_building(&mut self, building_type: BuildingType) -> bool {
        if let Some(count) = self.owned.get_mut(&building_type) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    self.owned.remove(&building_type);
                }
                return true;
            }
        }
        false
    }

    /// 获取已拥有建筑数量
    pub fn owned_count(&self, building_type: BuildingType) -> u32 {
        *self.owned.get(&building_type).unwrap_or(&0)
    }

    /// 放置建筑
    pub fn place(&mut self, building_type: BuildingType, x: usize, y: usize) {
        self.buildings.push(Building::new(building_type, x, y));
    }

    /// 获取指定位置的建筑
    pub fn get_at(&self, x: usize, y: usize) -> Option<&Building> {
        self.buildings.iter().find(|b| b.x == x && b.y == y)
    }

    /// 获取可放置的建筑列表
    pub fn get_placeable_buildings(&self) -> Vec<(BuildingType, u32)> {
        self.owned.iter().map(|(&bt, &count)| (bt, count)).collect()
    }

    /// 运行洒水器，返回浇水的地块数
    pub fn run_sprinklers(&mut self, grid: &mut Grid) -> usize {
        let mut count = 0;
        for building in &self.buildings {
            if building.building_type == BuildingType::SprinklerT1 {
                let range = building.building_type.range() as i32;
                for dy in -range..=range {
                    for dx in -range..=range {
                        if dx == 0 && dy == 0 {
                            continue; // 跳过建筑本身
                        }
                        let x = (building.x as i32 + dx) as usize;
                        let y = (building.y as i32 + dy) as usize;
                        if x < grid.width && y < grid.height {
                            if grid.water(x, y) {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        count
    }

    /// 运行祝尼魔小屋，返回收割的作物列表
    pub fn run_junimo_huts(&mut self, grid: &mut Grid, inventory: &mut Inventory) -> Vec<CropType> {
        let mut harvested = Vec::new();
        for building in &self.buildings {
            if building.building_type == BuildingType::JunimoHut {
                let range = building.building_type.range() as i32;
                for dy in -range..=range {
                    for dx in -range..=range {
                        let x = (building.x as i32 + dx) as usize;
                        let y = (building.y as i32 + dy) as usize;
                        if x < grid.width && y < grid.height {
                            if let Some(crop_type) = grid.harvest(x, y) {
                                inventory.add(super::types::ItemType::Crop(crop_type), 1);
                                harvested.push(crop_type);
                            }
                        }
                    }
                }
            }
        }
        harvested
    }

    /// 运行加工桶，制作果酱
    /// 从背包使用作物制作果酱，增加价值
    pub fn run_jam_makers(&mut self, _grid: &mut Grid, inventory: &mut Inventory) -> Vec<CropType> {
        let mut processed = Vec::new();
        use super::types::ItemType;

        // 获取背包中的作物列表（按价值排序，优先处理高价值作物）
        let mut crops: Vec<CropType> = inventory
            .list_crops()
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(crop, _)| crop)
            .collect();

        // 按出售价格降序排序（优先制作高价值果酱）
        crops.sort_by(|a, b| b.sell_price().cmp(&a.sell_price()));

        for building in &self.buildings {
            if building.building_type == BuildingType::JamMaker {
                // 尝试从背包找到一个作物来制作果酱
                for crop in &crops {
                    let item = ItemType::Crop(*crop);
                    if inventory.has_item(&item, 1) {
                        inventory.use_item(&item);
                        inventory.add(ItemType::Jelly(*crop), 1);
                        processed.push(*crop);
                        break; // 每个加工桶每次只处理一个
                    }
                }
            }
        }
        processed
    }

    /// 获取所有建筑
    pub fn all(&self) -> &[Building] {
        &self.buildings
    }
}

impl Default for BuildingManager {
    fn default() -> Self {
        Self::new()
    }
}
