// 网格系统

use super::types::{Crop, CropType, SoilState, Tile};
use serde::{Deserialize, Serialize};

/// 农场网格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub tiles: Vec<Vec<Tile>>,
    pub width: usize,
    pub height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![vec![Tile::default(); width]; height];
        Self {
            tiles,
            width,
            height,
        }
    }

    /// 获取地块
    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y).and_then(|row| row.get(x))
    }

    /// 获取可变地块
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(y).and_then(|row| row.get_mut(x))
    }

    /// 耕地
    pub fn till(&mut self, x: usize, y: usize) -> bool {
        if let Some(tile) = self.get_mut(x, y) {
            if tile.soil == SoilState::Grass {
                tile.soil = SoilState::Tilled;
                return true;
            }
        }
        false
    }

    /// 浇水
    pub fn water(&mut self, x: usize, y: usize) -> bool {
        if let Some(tile) = self.get_mut(x, y) {
            if tile.soil == SoilState::Tilled {
                tile.soil = SoilState::Watered;
                if let Some(crop) = &mut tile.crop {
                    crop.watered = true;
                }
                return true;
            } else if tile.soil == SoilState::Watered {
                // 已经浇过水了，只给作物浇水
                if let Some(crop) = &mut tile.crop {
                    crop.watered = true;
                    return true;
                }
            }
        }
        false
    }

    /// 种植
    pub fn plant(&mut self, x: usize, y: usize, crop_type: CropType) -> bool {
        if let Some(tile) = self.get_mut(x, y) {
            if tile.crop.is_none() && tile.soil != SoilState::Grass {
                let mut crop = Crop::new(crop_type);
                // 如果土壤已浇水，作物也被浇水
                if tile.soil == SoilState::Watered {
                    crop.watered = true;
                }
                tile.crop = Some(crop);
                return true;
            }
        }
        false
    }

    /// 收获
    pub fn harvest(&mut self, x: usize, y: usize) -> Option<CropType> {
        if let Some(tile) = self.get_mut(x, y) {
            if let Some(crop) = &tile.crop {
                if crop.is_mature() {
                    let crop_type = crop.crop_type;
                    tile.crop = None;
                    tile.soil = SoilState::Tilled; // 收获后变回耕地
                    return Some(crop_type);
                }
            }
        }
        None
    }

    /// 清除作物（不收获）
    pub fn clear(&mut self, x: usize, y: usize) -> bool {
        if let Some(tile) = self.get_mut(x, y) {
            if tile.crop.is_some() {
                tile.crop = None;
                return true;
            }
        }
        false
    }

    /// 扩展网格
    pub fn expand(&mut self, new_width: usize, new_height: usize) {
        // 扩展每一行
        for row in &mut self.tiles {
            while row.len() < new_width {
                row.push(Tile::default());
            }
        }

        // 添加新行
        while self.tiles.len() < new_height {
            self.tiles.push(vec![Tile::default(); new_width]);
        }

        self.width = new_width;
        self.height = new_height;
    }
}
