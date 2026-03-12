// 游戏类型定义

use serde::{Deserialize, Serialize};

/// 气候带
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Biome {
    /// 春之平原 (IP末位 % 3 == 0)
    SpringPlains,
    /// 夏之海岛 (IP末位 % 3 == 1)
    SummerIsles,
    /// 秋之矿山 (IP末位 % 3 == 2)
    AutumnHills,
}

impl Biome {
    /// 根据 IP 最后一位生成气候带
    pub fn from_ip(last_octet: u8) -> Self {
        match last_octet % 3 {
            0 => Biome::SpringPlains,
            1 => Biome::SummerIsles,
            _ => Biome::AutumnHills,
        }
    }

    /// 气候名称
    pub fn name(&self) -> &'static str {
        match self {
            Biome::SpringPlains => "春之平原",
            Biome::SummerIsles => "夏之海岛",
            Biome::AutumnHills => "秋之矿山",
        }
    }

    /// 特产作物
    pub fn specialty_crops(&self) -> Vec<CropType> {
        match self {
            Biome::SpringPlains => vec![CropType::Parsnip, CropType::Potato, CropType::GreenBean],
            Biome::SummerIsles => vec![CropType::Blueberry, CropType::Melon, CropType::HotPepper],
            Biome::AutumnHills => vec![CropType::Pumpkin, CropType::Cranberry, CropType::Yam],
        }
    }
}

/// 作物类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CropType {
    Parsnip,   // 防风草
    Potato,    // 土豆
    GreenBean, // 绿豆
    Blueberry, // 蓝莓
    Melon,     // 甜瓜
    HotPepper, // 辣椒
    Pumpkin,   // 南瓜
    Cranberry, // 蔓越莓
    Yam,       // 红薯
}

impl CropType {
    /// 作物名称
    pub fn name(&self) -> &'static str {
        match self {
            CropType::Parsnip => "防风草",
            CropType::Potato => "土豆",
            CropType::GreenBean => "绿豆",
            CropType::Blueberry => "蓝莓",
            CropType::Melon => "甜瓜",
            CropType::HotPepper => "辣椒",
            CropType::Pumpkin => "南瓜",
            CropType::Cranberry => "蔓越莓",
            CropType::Yam => "红薯",
        }
    }

    /// 作物图标
    pub fn icon(&self) -> &'static str {
        match self {
            CropType::Parsnip => "🥕",
            CropType::Potato => "🥔",
            CropType::GreenBean => "🫛",
            CropType::Blueberry => "🫐",
            CropType::Melon => "🍈",
            CropType::HotPepper => "🌶",
            CropType::Pumpkin => "🎃",
            CropType::Cranberry => "🫐",
            CropType::Yam => "🍠",
        }
    }

    /// 生长天数
    pub fn grow_days(&self) -> u32 {
        match self {
            CropType::Parsnip => 4,
            CropType::Potato => 6,
            CropType::GreenBean => 5,
            CropType::Blueberry => 7,
            CropType::Melon => 8,
            CropType::HotPepper => 5,
            CropType::Pumpkin => 10,
            CropType::Cranberry => 6,
            CropType::Yam => 7,
        }
    }

    /// 种子价格
    pub fn seed_price(&self) -> u32 {
        match self {
            CropType::Parsnip => 20,
            CropType::Potato => 50,
            CropType::GreenBean => 60,
            CropType::Blueberry => 80,
            CropType::Melon => 100,
            CropType::HotPepper => 40,
            CropType::Pumpkin => 150,
            CropType::Cranberry => 90,
            CropType::Yam => 70,
        }
    }

    /// 出售价格
    pub fn sell_price(&self) -> u32 {
        match self {
            CropType::Parsnip => 35,
            CropType::Potato => 80,
            CropType::GreenBean => 90,
            CropType::Blueberry => 120,
            CropType::Melon => 180,
            CropType::HotPepper => 70,
            CropType::Pumpkin => 250,
            CropType::Cranberry => 140,
            CropType::Yam => 110,
        }
    }
}

/// 作物生长阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropStage {
    Seed,    // 种子
    Sprout,  // 幼苗
    Growing, // 生长中
    Mature,  // 成熟
}

/// 作物
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crop {
    pub crop_type: CropType,
    pub growth: u32,     // 当前生长值
    pub max_growth: u32, // 最大生长值
    pub watered: bool,   // 是否已浇水
}

impl Crop {
    pub fn new(crop_type: CropType) -> Self {
        let grow_days = crop_type.grow_days();
        Self {
            crop_type,
            growth: 0,
            max_growth: grow_days * 10, // 每天10生长值
            watered: false,
        }
    }

    /// 生长
    pub fn grow(&mut self) -> bool {
        if self.watered && self.growth < self.max_growth {
            self.growth += 1;
            self.watered = false;
            true
        } else {
            false
        }
    }

    /// 是否成熟
    pub fn is_mature(&self) -> bool {
        self.growth >= self.max_growth
    }

    /// 生长进度 (0.0 - 1.0)
    pub fn growth_progress(&self) -> f32 {
        if self.max_growth == 0 {
            0.0
        } else {
            self.growth as f32 / self.max_growth as f32
        }
    }

    /// 当前阶段
    pub fn stage(&self) -> CropStage {
        let progress = self.growth_progress();
        if progress >= 1.0 {
            CropStage::Mature
        } else if progress >= 0.6 {
            CropStage::Growing
        } else if progress >= 0.3 {
            CropStage::Sprout
        } else {
            CropStage::Seed
        }
    }

    /// 图标
    pub fn icon(&self) -> &'static str {
        match self.stage() {
            CropStage::Seed => "🌱",
            CropStage::Sprout => "🌿",
            CropStage::Growing => self.crop_type.icon(),
            CropStage::Mature => self.crop_type.icon(),
        }
    }
}

/// 土壤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoilState {
    Grass,   // 草地
    Tilled,  // 已耕地
    Watered, // 已浇水
}

/// 地块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub soil: SoilState,
    pub crop: Option<Crop>,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            soil: SoilState::Grass,
            crop: None,
        }
    }
}

/// 天气
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weather {
    Sunny,  // 晴天
    Cloudy, // 多云
    Rainy,  // 下雨
    Stormy, // 暴风雨
}

impl Weather {
    pub fn name(&self) -> &'static str {
        match self {
            Weather::Sunny => "晴天",
            Weather::Cloudy => "多云",
            Weather::Rainy => "下雨",
            Weather::Stormy => "暴风雨",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Weather::Sunny => "☀️",
            Weather::Cloudy => "⛅",
            Weather::Rainy => "🌧️",
            Weather::Stormy => "⛈️",
        }
    }

    /// 是否自动浇水
    pub fn auto_waters(&self) -> bool {
        matches!(self, Weather::Rainy | Weather::Stormy)
    }
}

/// 季节
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn name(&self) -> &'static str {
        match self {
            Season::Spring => "春",
            Season::Summer => "夏",
            Season::Autumn => "秋",
            Season::Winter => "冬",
        }
    }
}

/// 游戏时间
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTime {
    pub year: u32,
    pub season: Season,
    pub day: u32, // 1-28
}

impl GameTime {
    pub fn new() -> Self {
        Self {
            year: 1,
            season: Season::Spring,
            day: 1,
        }
    }

    /// 推进一天
    pub fn advance_day(&mut self) {
        self.day += 1;
        if self.day > 28 {
            self.day = 1;
            self.advance_season();
        }
    }

    /// 推进一季
    fn advance_season(&mut self) {
        self.season = match self.season {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => {
                self.year += 1;
                Season::Spring
            }
        };
    }

    pub fn season_name(&self) -> &'static str {
        self.season.name()
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

/// 料理类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DishType {
    PotatoPancake,     // 土豆煎饼
    MelonJuice,        // 甜瓜汁
    CompleteBreakfast, // 全家桶早餐
    PumpkinPie,        // 南瓜派
    YamRoast,          // 红薯烤物
    BlueberryMuffin,   // 蓝莓松饼
    PepperSteak,       // 辣椒炒肉
    CranberrySauce,    // 蔓越莓酱
    ParsnipSoup,       // 防风草根汤
    GreenBeanSoup,     // 绿豆汤
}

impl DishType {
    /// 料理名称
    pub fn name(&self) -> &'static str {
        match self {
            DishType::PotatoPancake => "土豆煎饼",
            DishType::MelonJuice => "甜瓜汁",
            DishType::CompleteBreakfast => "全家桶早餐",
            DishType::PumpkinPie => "南瓜派",
            DishType::YamRoast => "红薯烤物",
            DishType::BlueberryMuffin => "蓝莓松饼",
            DishType::PepperSteak => "辣椒炒肉",
            DishType::CranberrySauce => "蔓越莓酱",
            DishType::ParsnipSoup => "防风草根汤",
            DishType::GreenBeanSoup => "绿豆汤",
        }
    }

    /// 料理图标
    pub fn icon(&self) -> &'static str {
        match self {
            DishType::PotatoPancake => "🥞",
            DishType::MelonJuice => "🥤",
            DishType::CompleteBreakfast => "🍳",
            DishType::PumpkinPie => "🥧",
            DishType::YamRoast => "🍠",
            DishType::BlueberryMuffin => "🧁",
            DishType::PepperSteak => "🥩",
            DishType::CranberrySauce => "🫙",
            DishType::ParsnipSoup => "🍲",
            DishType::GreenBeanSoup => "🥣",
        }
    }

    /// 出售价格
    pub fn sell_price(&self) -> u32 {
        match self {
            DishType::PotatoPancake => 150,
            DishType::MelonJuice => 180,
            DishType::CompleteBreakfast => 500,
            DishType::PumpkinPie => 400,
            DishType::YamRoast => 250,
            DishType::BlueberryMuffin => 200,
            DishType::PepperSteak => 300,
            DishType::CranberrySauce => 220,
            DishType::ParsnipSoup => 120,
            DishType::GreenBeanSoup => 100,
        }
    }
}

/// 物品类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Crop(CropType),
    Seed(CropType),
    Jelly(CropType), // 果酱
    Dish(DishType),  // 料理
}

impl ItemType {
    /// 物品名称
    pub fn name(&self) -> String {
        match self {
            ItemType::Crop(crop) => crop.name().to_string(),
            ItemType::Seed(crop) => format!("{}种子", crop.name()),
            ItemType::Jelly(crop) => format!("{}果酱", crop.name()),
            ItemType::Dish(dish) => dish.name().to_string(),
        }
    }

    /// 物品图标
    pub fn icon(&self) -> &'static str {
        match self {
            ItemType::Crop(crop) => crop.icon(),
            ItemType::Seed(crop) => crop.icon(),
            ItemType::Jelly(_) => "🍯",
            ItemType::Dish(dish) => dish.icon(),
        }
    }

    /// 出售价格
    pub fn sell_price(&self) -> u32 {
        match self {
            ItemType::Crop(crop) => crop.sell_price(),
            ItemType::Seed(crop) => crop.seed_price() / 2,
            ItemType::Jelly(crop) => (crop.sell_price() as f64 * 1.5) as u32,
            ItemType::Dish(dish) => dish.sell_price(),
        }
    }
}

/// 建筑类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    SprinklerT1, // 洒水器 T1
    JunimoHut,   // 祝尼魔小屋
    JamMaker,    // 加工桶
    ShippingBox, // 出货箱
    Kitchen,     // 厨房
}

impl BuildingType {
    pub fn name(&self) -> &'static str {
        match self {
            BuildingType::SprinklerT1 => "洒水器",
            BuildingType::JunimoHut => "祝尼魔小屋",
            BuildingType::JamMaker => "加工桶",
            BuildingType::ShippingBox => "出货箱",
            BuildingType::Kitchen => "厨房",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            BuildingType::SprinklerT1 => "💦",
            BuildingType::JunimoHut => "🏠",
            BuildingType::JamMaker => "🫙",
            BuildingType::ShippingBox => "📦",
            BuildingType::Kitchen => "🍳",
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            BuildingType::SprinklerT1 => 500,
            BuildingType::JunimoHut => 2000,
            BuildingType::JamMaker => 300,
            BuildingType::ShippingBox => 100,
            BuildingType::Kitchen => 1500,
        }
    }

    /// 建筑影响范围
    pub fn range(&self) -> u32 {
        match self {
            BuildingType::SprinklerT1 => 1, // 周围1格
            BuildingType::JunimoHut => 2,   // 周围2格
            _ => 0,
        }
    }

    /// 建筑描述
    pub fn description(&self) -> &'static str {
        match self {
            BuildingType::SprinklerT1 => "自动浇水周围4格",
            BuildingType::JunimoHut => "自动收割周围作物",
            BuildingType::JamMaker => "将作物制作成果酱",
            BuildingType::ShippingBox => "快速出售作物",
            BuildingType::Kitchen => "烹饪美味料理",
        }
    }
}

/// 建筑
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub building_type: BuildingType,
    pub x: usize,
    pub y: usize,
}

impl Building {
    pub fn new(building_type: BuildingType, x: usize, y: usize) -> Self {
        Self {
            building_type,
            x,
            y,
        }
    }

    pub fn name(&self) -> &'static str {
        self.building_type.name()
    }

    pub fn icon(&self) -> &'static str {
        self.building_type.icon()
    }

    /// 检查坐标是否在影响范围内
    pub fn in_range(&self, x: usize, y: usize) -> bool {
        let range = self.building_type.range() as i32;
        let dx = (x as i32 - self.x as i32).abs();
        let dy = (y as i32 - self.y as i32).abs();
        dx <= range && dy <= range
    }
}
