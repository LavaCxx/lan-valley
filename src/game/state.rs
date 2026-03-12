// 游戏状态

use super::building::BuildingManager;
use super::grid::Grid;
use super::inventory::Inventory;
use super::save::SaveManager;
use super::types::{Biome, BuildingType, CropType, GameTime, Weather};
use serde::{Deserialize, Serialize};

/// 游戏模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// 正常游戏
    Normal,
    /// 商店
    Shop,
    /// 背包
    Inventory,
    /// 种植选择
    PlantSelect,
    /// 建筑商店
    BuildingShop,
    /// 建筑放置
    BuildingPlace,
    /// 烹饪菜单
    Cooking,
}

/// 当前工具（用于鼠标操作）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tool {
    /// 耕地
    Till,
    /// 浇水
    Water,
    /// 种植
    Plant,
    /// 收获
    Harvest,
}

impl Tool {
    pub fn icon(&self) -> &'static str {
        match self {
            Tool::Till => "🔧",
            Tool::Water => "💧",
            Tool::Plant => "🌱",
            Tool::Harvest => "🌾",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tool::Till => "耕地",
            Tool::Water => "浇水",
            Tool::Plant => "种植",
            Tool::Harvest => "收获",
        }
    }

    pub fn all() -> Vec<Tool> {
        vec![Tool::Till, Tool::Water, Tool::Plant, Tool::Harvest]
    }
}

/// 游戏状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// 农场网格
    pub grid: Grid,
    /// 背包
    pub inventory: Inventory,
    /// 建筑管理器
    pub buildings: BuildingManager,
    /// 游戏时间
    pub time: GameTime,
    /// 气候带
    pub biome: Biome,
    /// 天气
    pub weather: Weather,
    /// 玩家名称
    pub player_name: String,
    /// 光标位置
    pub cursor: (usize, usize),
    /// 是否暂停
    pub paused: bool,
    /// 游戏刻计数器
    tick_counter: u32,
    /// 当前游戏模式
    pub mode: GameMode,
    /// 商店/背包选择索引
    pub menu_index: usize,
    /// 日志消息
    pub logs: Vec<String>,
    /// 待放置的建筑类型
    pub pending_building: Option<BuildingType>,
    /// 当前工具（鼠标操作用）
    pub current_tool: Tool,
    /// 自动保存计数器
    #[serde(skip)]
    auto_save_counter: u32,
}

impl GameState {
    /// 创建新游戏
    pub fn new(player_name: String, ip_last_octet: u8) -> Self {
        let biome = Biome::from_ip(ip_last_octet);

        let mut state = Self {
            grid: Grid::new(5, 5), // 初始 5x5 农场
            inventory: Inventory::new(),
            buildings: BuildingManager::new(),
            time: GameTime::new(),
            biome,
            weather: Weather::Sunny,
            player_name,
            cursor: (0, 0),
            paused: false,
            tick_counter: 0,
            mode: GameMode::Normal,
            menu_index: 0,
            logs: Vec::new(),
            pending_building: None,
            current_tool: Tool::Till,
            auto_save_counter: 0,
        };

        // 初始赠送一些种子
        let specialty = biome.specialty_crops();
        if let Some(&crop) = specialty.first() {
            state.inventory.add(super::types::ItemType::Seed(crop), 10);
            state.log(format!(
                "欢迎来到{}！赠送你10颗{}种子",
                biome.name(),
                crop.name()
            ));
        }

        state
    }

    /// 从存档加载或创建新游戏
    pub fn load_or_new(player_name: String, ip_last_octet: u8) -> Self {
        let save_manager = match SaveManager::new() {
            Ok(sm) => sm,
            Err(_) => return GameState::new(player_name, ip_last_octet),
        };

        if save_manager.exists() {
            match save_manager.load() {
                Ok(mut state) => {
                    state.log("读取存档成功！".to_string());
                    state
                }
                Err(e) => {
                    let mut state = GameState::new(player_name, ip_last_octet);
                    state.log(format!("读取存档失败: {}，创建新游戏", e));
                    state
                }
            }
        } else {
            GameState::new(player_name, ip_last_octet)
        }
    }

    /// 保存游戏
    pub fn save(&self) -> bool {
        match SaveManager::new() {
            Ok(save_manager) => match save_manager.save(self) {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("保存失败: {}", e);
                    false
                }
            },
            Err(_) => false,
        }
    }

    /// 添加日志
    pub fn log(&mut self, message: String) {
        let timestamp = format!("[{:02}:{:02}]", self.time.day, self.tick_counter / 10);
        self.logs.push(format!("{} {}", timestamp, message));
        if self.logs.len() > 10 {
            self.logs.remove(0);
        }
    }

    /// 游戏主循环更新
    pub fn update(&mut self) {
        if self.paused || self.mode != GameMode::Normal {
            return;
        }

        self.tick_counter += 1;

        // 每 6000 刻 = 10 分钟 = 1 游戏天 (tick_rate = 100ms)
        if self.tick_counter >= 6000 {
            self.tick_counter = 0;
            self.advance_day();
        }

        // 自动保存（每 600 刻 = 10 游戏天）
        self.auto_save_counter += 1;
        if self.auto_save_counter >= 600 {
            self.auto_save_counter = 0;
            if self.save() {
                self.log("自动保存成功".to_string());
            }
        }
    }

    /// 推进一天
    pub fn advance_day(&mut self) {
        // 更新天气
        self.update_weather();

        // 下雨天自动浇水
        if self.weather.auto_waters() {
            self.water_all();
            self.log(format!("{}自动浇灌了所有作物", self.weather.name()));
        }

        // 洒水器自动浇水
        let sprinkler_count = self.buildings.run_sprinklers(&mut self.grid);
        if sprinkler_count > 0 {
            self.log(format!("洒水器浇灌了{}块地", sprinkler_count));
        }

        // 祝尼魔小屋自动收割
        let harvested = self
            .buildings
            .run_junimo_huts(&mut self.grid, &mut self.inventory);
        if !harvested.is_empty() {
            self.log(format!("祝尼魔收割了{}株作物", harvested.len()));
        }

        // 加工桶制作果酱
        let processed = self
            .buildings
            .run_jam_makers(&mut self.grid, &mut self.inventory);
        if !processed.is_empty() {
            self.log(format!("加工桶制作了{}个果酱", processed.len()));
        }

        // 作物生长
        let mut grown_count = 0;
        let mut mature_count = 0;
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                if let Some(tile) = self.grid.get_mut(x, y) {
                    if let Some(crop) = &mut tile.crop {
                        if crop.grow() {
                            grown_count += 1;
                            if crop.is_mature() {
                                mature_count += 1;
                            }
                        }
                    }
                }
            }
        }

        if grown_count > 0 {
            self.log(format!("{}株作物生长了", grown_count));
        }
        if mature_count > 0 {
            self.log(format!("{}株作物成熟了！", mature_count));
        }

        // 推进时间
        self.time.advance_day();
    }

    /// 更新天气
    fn update_weather(&mut self) {
        use rand::Rng;
        let mut rng = rand::rng();
        let roll: u32 = rng.random_range(0..100);

        self.weather = if roll < 50 {
            Weather::Sunny
        } else if roll < 75 {
            Weather::Cloudy
        } else if roll < 95 {
            Weather::Rainy
        } else {
            Weather::Stormy
        };
    }

    /// 浇水所有地块
    fn water_all(&mut self) {
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                self.grid.water(x, y);
            }
        }
    }

    /// 移动光标
    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        match self.mode {
            GameMode::Normal => {
                let new_x = (self.cursor.0 as i32 + dx)
                    .max(0)
                    .min(self.grid.width as i32 - 1) as usize;
                let new_y = (self.cursor.1 as i32 + dy)
                    .max(0)
                    .min(self.grid.height as i32 - 1) as usize;
                self.cursor = (new_x, new_y);
            }
            GameMode::Shop
            | GameMode::Inventory
            | GameMode::PlantSelect
            | GameMode::BuildingShop
            | GameMode::Cooking => {
                // 菜单导航
                let max_items = self.get_menu_items().len();
                if max_items > 0 {
                    let new_index = if dy < 0 {
                        self.menu_index.saturating_sub(1)
                    } else {
                        (self.menu_index + 1).min(max_items - 1)
                    };
                    self.menu_index = new_index;
                }
            }
            GameMode::BuildingPlace => {
                // 建筑放置模式下移动光标
                let new_x = (self.cursor.0 as i32 + dx)
                    .max(0)
                    .min(self.grid.width as i32 - 1) as usize;
                let new_y = (self.cursor.1 as i32 + dy)
                    .max(0)
                    .min(self.grid.height as i32 - 1) as usize;
                self.cursor = (new_x, new_y);
            }
        }
    }

    /// 设置光标位置（用于鼠标点击）
    pub fn set_cursor(&mut self, x: usize, y: usize) {
        if x < self.grid.width && y < self.grid.height {
            self.cursor = (x, y);
        }
    }

    /// 使用当前工具
    pub fn use_tool(&mut self) {
        if self.mode != GameMode::Normal {
            return;
        }
        match self.current_tool {
            Tool::Till => {
                self.till();
            }
            Tool::Water => {
                self.water();
            }
            Tool::Plant => {
                // 种植第一个可用种子
                let seeds = self.inventory.list_seeds();
                if let Some((crop, _)) = seeds.first() {
                    if self.inventory.use_seed(*crop) {
                        if !self.grid.plant(self.cursor.0, self.cursor.1, *crop) {
                            self.inventory.add(super::types::ItemType::Seed(*crop), 1);
                        } else {
                            self.log(format!("种下了{}", crop.name()));
                        }
                    }
                } else {
                    self.log("没有种子！".to_string());
                }
            }
            Tool::Harvest => {
                self.harvest();
            }
        }
    }

    /// 选择工具
    pub fn select_tool(&mut self, tool: Tool) {
        self.current_tool = tool;
    }

    /// 智能操作（根据地块状态自动选择操作）
    pub fn smart_action(&mut self) {
        if self.mode != GameMode::Normal {
            return;
        }

        if let Some(tile) = self.grid.get(self.cursor.0, self.cursor.1) {
            // 优先收获成熟作物
            if let Some(crop) = &tile.crop {
                if crop.is_mature() {
                    self.harvest();
                    return;
                }
            }

            // 没有作物时
            if tile.crop.is_none() {
                match tile.soil {
                    super::types::SoilState::Grass => {
                        self.till();
                    }
                    super::types::SoilState::Tilled => {
                        self.water();
                    }
                    super::types::SoilState::Watered => {
                        // 自动种植第一个可用种子
                        let seeds = self.inventory.list_seeds();
                        if let Some((crop, _)) = seeds.first() {
                            if self.inventory.use_seed(*crop) {
                                if !self.grid.plant(self.cursor.0, self.cursor.1, *crop) {
                                    self.inventory.add(super::types::ItemType::Seed(*crop), 1);
                                } else {
                                    self.log(format!("种下了{}", crop.name()));
                                }
                            }
                        }
                    }
                }
            } else if let Some(crop) = &tile.crop {
                // 有作物但未成熟，检查是否需要浇水
                if !crop.watered && tile.soil != super::types::SoilState::Watered {
                    self.water();
                }
            }
        }
    }

    /// 获取农场升级费用
    pub fn farm_upgrade_cost(&self) -> u32 {
        let current_size = self.grid.width.max(self.grid.height);
        // 每次升级费用 = 当前大小 * 200
        (current_size as u32) * 200
    }

    /// 检查是否可以升级农场
    pub fn can_upgrade_farm(&self) -> bool {
        let max_size = 12;
        self.grid.width < max_size && self.grid.height < max_size
    }

    /// 升级农场（扩大 1x1）
    pub fn upgrade_farm(&mut self) -> bool {
        if !self.can_upgrade_farm() {
            self.log("农场已达最大尺寸！".to_string());
            return false;
        }

        let cost = self.farm_upgrade_cost();
        if self.inventory.gold < cost {
            self.log(format!("金币不足！需要 {} 金币", cost));
            return false;
        }

        self.inventory.gold -= cost;
        let new_width = (self.grid.width + 1).min(12);
        let new_height = (self.grid.height + 1).min(12);

        // 扩展网格
        self.grid.expand(new_width, new_height);
        self.log(format!(
            "农场升级成功！现在 {}x{}",
            self.grid.width, self.grid.height
        ));
        true
    }

    /// 获取当前菜单项
    pub fn get_menu_items(&self) -> Vec<MenuItem> {
        match self.mode {
            GameMode::Shop => {
                let mut items = Vec::new();

                // 农场升级选项
                if self.can_upgrade_farm() {
                    let cost = self.farm_upgrade_cost();
                    let current = format!("{}x{}", self.grid.width, self.grid.height);
                    items.push(MenuItem {
                        label: format!("🌱 升级农场 {} → {}x{} ({}G)", current, self.grid.width + 1, self.grid.height + 1, cost),
                        crop: None,
                        building: None,
                    });
                } else {
                    items.push(MenuItem {
                        label: format!("🌱 农场已达最大 ({}x{})", self.grid.width, self.grid.height),
                        crop: None,
                        building: None,
                    });
                }

                // 所有可购买的种子
                items.extend(vec![
                    MenuItem {
                        label: format!("防风草种子 (20G)"),
                        crop: Some(CropType::Parsnip),
                        building: None,
                    },
                    MenuItem {
                        label: format!("土豆种子 (50G)"),
                        crop: Some(CropType::Potato),
                        building: None,
                    },
                    MenuItem {
                        label: format!("绿豆种子 (60G)"),
                        crop: Some(CropType::GreenBean),
                        building: None,
                    },
                    MenuItem {
                        label: format!("蓝莓种子 (80G)"),
                        crop: Some(CropType::Blueberry),
                        building: None,
                    },
                    MenuItem {
                        label: format!("甜瓜种子 (100G)"),
                        crop: Some(CropType::Melon),
                        building: None,
                    },
                    MenuItem {
                        label: format!("辣椒种子 (40G)"),
                        crop: Some(CropType::HotPepper),
                        building: None,
                    },
                    MenuItem {
                        label: format!("南瓜种子 (150G)"),
                        crop: Some(CropType::Pumpkin),
                        building: None,
                    },
                    MenuItem {
                        label: format!("蔓越莓种子 (90G)"),
                        crop: Some(CropType::Cranberry),
                        building: None,
                    },
                    MenuItem {
                        label: format!("红薯种子 (70G)"),
                        crop: Some(CropType::Yam),
                        building: None,
                    },
                ]);
                items
            }
            GameMode::BuildingShop => {
                // 建筑商店
                vec![
                    MenuItem {
                        label: format!("💦 洒水器 (500G) - 自动浇水周围4格"),
                        crop: None,
                        building: Some(BuildingType::SprinklerT1),
                    },
                    MenuItem {
                        label: format!("🏠 祝尼魔小屋 (2000G) - 自动收割周围作物"),
                        crop: None,
                        building: Some(BuildingType::JunimoHut),
                    },
                    MenuItem {
                        label: format!("🫙 加工桶 (300G) - 制作果酱"),
                        crop: None,
                        building: Some(BuildingType::JamMaker),
                    },
                    MenuItem {
                        label: format!("📦 出货箱 (100G) - 快速出售作物"),
                        crop: None,
                        building: Some(BuildingType::ShippingBox),
                    },
                    MenuItem {
                        label: format!("🍳 厨房 (1500G) - 烹饪料理"),
                        crop: None,
                        building: Some(BuildingType::Kitchen),
                    },
                ]
            }
            GameMode::Inventory => {
                // 背包中的作物
                let crops = self.inventory.list_crops();
                crops
                    .iter()
                    .map(|(crop, count)| MenuItem {
                        label: format!("{} {} ({}个)", crop.icon(), crop.name(), count),
                        crop: Some(*crop),
                        building: None,
                    })
                    .collect()
            }
            GameMode::PlantSelect => {
                // 背包中的种子
                let seeds = self.inventory.list_seeds();
                seeds
                    .iter()
                    .map(|(crop, count)| MenuItem {
                        label: format!("{} {}种子 ({}个)", crop.icon(), crop.name(), count),
                        crop: Some(*crop),
                        building: None,
                    })
                    .collect()
            }
            GameMode::Normal | GameMode::BuildingPlace => vec![],
            GameMode::Cooking => {
                // 可制作的料理列表（按难度排序）
                use super::types::DishType;
                let all_dishes = [
                    // 基础料理（使用作物）
                    DishType::PotatoPancake,
                    DishType::MelonJuice,
                    DishType::PumpkinPie,
                    DishType::YamRoast,
                    DishType::BlueberryMuffin,
                    DishType::PepperSteak,
                    DishType::CranberrySauce,
                    DishType::ParsnipSoup,
                    DishType::GreenBeanSoup,
                    // 高级料理（需要其他料理，强制跨气候带交换）
                    DishType::CompleteBreakfast,
                ];

                all_dishes
                    .iter()
                    .map(|&dish| {
                        let ingredients = Self::dish_ingredients(dish);
                        let can_make = self.can_cook(dish);
                        let status = if can_make { "✓" } else { "✗" };
                        MenuItem {
                            label: format!(
                                "{} {} {} ({}G) - 需要: {}",
                                status,
                                dish.icon(),
                                dish.name(),
                                dish.sell_price(),
                                Self::format_ingredients(&ingredients)
                            ),
                            crop: None,
                            building: None,
                        }
                    })
                    .collect()
            }
        }
    }

    /// 获取料理所需食材
    fn dish_ingredients(dish: super::types::DishType) -> Vec<(super::types::ItemType, u32)> {
        use super::types::DishType;
        use super::types::ItemType;
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

    /// 格式化食材列表
    fn format_ingredients(ingredients: &[(super::types::ItemType, u32)]) -> String {
        ingredients
            .iter()
            .map(|(item, count)| format!("{}x{}", item.name(), count))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// 检查是否可以烹饪
    fn can_cook(&self, dish: super::types::DishType) -> bool {
        let ingredients = Self::dish_ingredients(dish);
        for (item, count) in ingredients {
            let have = self.inventory.count(&item);
            if have < count {
                return false;
            }
        }
        true
    }

    /// 烹饪料理
    fn cook(&mut self, dish: super::types::DishType) -> bool {
        let ingredients = Self::dish_ingredients(dish);

        // 检查食材
        for (item, count) in &ingredients {
            let have = self.inventory.count(&item);
            if have < *count {
                self.log(format!("食材不足！需要{}x{}", item.name(), count));
                return false;
            }
        }

        // 消耗食材
        for (item, count) in ingredients {
            for _ in 0..count {
                self.inventory.use_item(&item);
            }
        }

        // 添加料理
        self.inventory.add(super::types::ItemType::Dish(dish), 1);
        self.log(format!("制作了{}！", dish.name()));
        true
    }

    /// 确认菜单选择
    pub fn confirm_menu(&mut self) {
        match self.mode {
            GameMode::Shop => {
                let items = self.get_menu_items();
                if let Some(item) = items.get(self.menu_index) {
                    // 第一项是农场升级
                    if self.menu_index == 0 && item.crop.is_none() {
                        self.upgrade_farm();
                    } else if let Some(crop) = item.crop {
                        if self.inventory.buy_seed(crop, 1) {
                            self.log(format!("购买了{}种子", crop.name()));
                        } else {
                            self.log("金币不足！".to_string());
                        }
                    }
                }
            }
            GameMode::BuildingShop => {
                let items = self.get_menu_items();
                if let Some(item) = items.get(self.menu_index) {
                    if let Some(building_type) = item.building {
                        let price = building_type.price();
                        if self.inventory.spend_gold(price) {
                            self.buildings.buy_building(building_type);
                            self.log(format!("购买了{}！按B放置", building_type.name()));
                        } else {
                            self.log("金币不足！".to_string());
                        }
                    }
                }
            }
            GameMode::Inventory => {
                let items = self.get_menu_items();
                if let Some(item) = items.get(self.menu_index) {
                    if let Some(crop) = item.crop {
                        if self.inventory.sell_crop(crop, 1) {
                            self.log(format!("出售了{}，获得{}G", crop.name(), crop.sell_price()));
                        }
                    }
                }
            }
            GameMode::PlantSelect => {
                let items = self.get_menu_items();
                if let Some(item) = items.get(self.menu_index) {
                    if let Some(crop) = item.crop {
                        if self.plant(crop) {
                            self.log(format!("种下了{}", crop.name()));
                        }
                    }
                }
                self.mode = GameMode::Normal;
            }
            GameMode::Normal => {}
            GameMode::BuildingPlace => {
                // 放置建筑
                if let Some(building_type) = self.pending_building {
                    if self.place_building(building_type) {
                        self.buildings.use_owned_building(building_type);
                        self.log(format!("放置了{}", building_type.name()));
                        // 检查是否还有同类建筑
                        if self.buildings.owned_count(building_type) == 0 {
                            self.pending_building = None;
                            self.mode = GameMode::Normal;
                        }
                    }
                }
            }
            GameMode::Cooking => {
                // 烹饪料理（与菜单列表保持一致）
                use super::types::DishType;
                let all_dishes = [
                    DishType::PotatoPancake,
                    DishType::MelonJuice,
                    DishType::PumpkinPie,
                    DishType::YamRoast,
                    DishType::BlueberryMuffin,
                    DishType::PepperSteak,
                    DishType::CranberrySauce,
                    DishType::ParsnipSoup,
                    DishType::GreenBeanSoup,
                    DishType::CompleteBreakfast,
                ];

                if let Some(&dish) = all_dishes.get(self.menu_index) {
                    self.cook(dish);
                }
            }
        }
    }

    /// 切换到商店模式
    pub fn open_shop(&mut self) {
        self.mode = GameMode::Shop;
        self.menu_index = 0;
    }

    /// 切换到建筑商店模式
    pub fn open_building_shop(&mut self) {
        self.mode = GameMode::BuildingShop;
        self.menu_index = 0;
    }

    /// 切换到背包模式
    pub fn open_inventory(&mut self) {
        self.mode = GameMode::Inventory;
        self.menu_index = 0;
    }

    /// 切换到种植选择模式
    pub fn open_plant_select(&mut self) {
        let seeds = self.inventory.list_seeds();
        if seeds.is_empty() {
            self.log("没有种子！".to_string());
        } else {
            self.mode = GameMode::PlantSelect;
            self.menu_index = 0;
        }
    }

    /// 开始放置建筑
    pub fn start_building_place(&mut self) {
        let placeable = self.buildings.get_placeable_buildings();
        if placeable.is_empty() {
            self.log("没有可放置的建筑！去商店购买吧".to_string());
        } else {
            // 选择第一个可放置的建筑
            let (building_type, _) = placeable[0];
            self.pending_building = Some(building_type);
            self.mode = GameMode::BuildingPlace;
            self.log(format!("选择位置放置{}", building_type.name()));
        }
    }

    /// 打开烹饪菜单
    pub fn open_cooking(&mut self) {
        // 检查是否有厨房
        let has_kitchen = self
            .buildings
            .all()
            .iter()
            .any(|b| b.building_type == BuildingType::Kitchen);
        if !has_kitchen {
            self.log("需要先建造厨房！".to_string());
            return;
        }
        self.mode = GameMode::Cooking;
        self.menu_index = 0;
    }

    /// 关闭菜单
    pub fn close_menu(&mut self) {
        self.mode = GameMode::Normal;
        self.pending_building = None;
    }

    /// 耕地
    pub fn till(&mut self) -> bool {
        if self.mode != GameMode::Normal {
            return false;
        }
        if self.grid.till(self.cursor.0, self.cursor.1) {
            self.log("耕地完成".to_string());
            true
        } else {
            false
        }
    }

    /// 浇水
    pub fn water(&mut self) -> bool {
        if self.mode != GameMode::Normal {
            return false;
        }
        if self.grid.water(self.cursor.0, self.cursor.1) {
            self.log("浇水完成".to_string());
            true
        } else {
            false
        }
    }

    /// 种植
    pub fn plant(&mut self, crop: CropType) -> bool {
        // 检查是否有种子
        if !self.inventory.use_seed(crop) {
            return false;
        }

        // 尝试种植
        if self.grid.plant(self.cursor.0, self.cursor.1, crop) {
            true
        } else {
            // 种植失败，返还种子
            self.inventory.add(super::types::ItemType::Seed(crop), 1);
            false
        }
    }

    /// 收获
    pub fn harvest(&mut self) -> Option<CropType> {
        if self.mode != GameMode::Normal {
            return None;
        }
        let crop = self.grid.harvest(self.cursor.0, self.cursor.1);
        if let Some(crop_type) = crop {
            self.inventory
                .add(super::types::ItemType::Crop(crop_type), 1);
            self.log(format!("收获了{}！", crop_type.name()));
        }
        crop
    }

    /// 购买建筑
    pub fn buy_building(&mut self, building_type: BuildingType) -> bool {
        let price = building_type.price();
        if self.inventory.spend_gold(price) {
            self.buildings.buy_building(building_type);
            self.log(format!("购买了{}！", building_type.name()));
            true
        } else {
            self.log("金币不足！".to_string());
            false
        }
    }

    /// 放置建筑
    pub fn place_building(&mut self, building_type: BuildingType) -> bool {
        // 检查位置是否可用
        if self.grid.get(self.cursor.0, self.cursor.1).is_none() {
            return false;
        }

        let tile = self.grid.get(self.cursor.0, self.cursor.1).unwrap();
        if tile.crop.is_some() {
            self.log("这个位置有作物！".to_string());
            return false;
        }

        if self
            .buildings
            .get_at(self.cursor.0, self.cursor.1)
            .is_some()
        {
            self.log("这个位置已有建筑！".to_string());
            return false;
        }

        self.buildings
            .place(building_type, self.cursor.0, self.cursor.1);

        // 出货箱特殊处理：放置时自动出售周围作物
        if building_type == BuildingType::ShippingBox {
            self.sell_nearby_crops();
        }

        true
    }

    /// 出货箱出售周围作物
    fn sell_nearby_crops(&mut self) {
        let range = 1;
        let mut sold_count = 0;
        let mut total_gold = 0;

        for dy in -range..=range {
            for dx in -range..=range {
                let x = (self.cursor.0 as i32 + dx) as usize;
                let y = (self.cursor.1 as i32 + dy) as usize;
                if x < self.grid.width && y < self.grid.height {
                    if let Some(crop_type) = self.grid.harvest(x, y) {
                        total_gold += crop_type.sell_price();
                        sold_count += 1;
                    }
                }
            }
        }

        if sold_count > 0 {
            self.inventory.earn_gold(total_gold);
            self.log(format!(
                "出货箱出售了{}个作物，获得{}G",
                sold_count, total_gold
            ));
        }
    }

    /// 获取当前地块信息
    pub fn current_tile_info(&self) -> Option<String> {
        if let Some(tile) = self.grid.get(self.cursor.0, self.cursor.1) {
            let mut info = format!("位置: ({}, {})\n", self.cursor.0, self.cursor.1);

            // 检查是否有建筑
            if let Some(building) = self.buildings.get_at(self.cursor.0, self.cursor.1) {
                info.push_str(&format!("建筑: {} {}\n", building.icon(), building.name()));
            }

            if let Some(crop) = &tile.crop {
                let progress = crop.growth_progress() * 100.0;
                let stage = crop.stage();
                info.push_str(&format!(
                    "作物: {} ({:.0}%)\n",
                    crop.crop_type.name(),
                    progress
                ));
                info.push_str(&format!("阶段: {:?}\n", stage));
                info.push_str(&format!(
                    "已浇水: {}",
                    if crop.watered { "是" } else { "否" }
                ));
            } else {
                info.push_str(&format!("土地状态: {:?}", tile.soil));
            }

            Some(info)
        } else {
            None
        }
    }

    /// 获取状态摘要
    pub fn status_summary(&self) -> String {
        format!(
            "第 {} 年 {}季 第 {} 天 | {} {} | 金币: {}",
            self.time.year,
            self.time.season_name(),
            self.time.day,
            self.weather.icon(),
            self.weather.name(),
            self.inventory.gold
        )
    }
}

/// 菜单项
pub struct MenuItem {
    pub label: String,
    pub crop: Option<CropType>,
    pub building: Option<BuildingType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_creation() {
        let state = GameState::new("测试玩家".to_string(), 55);
        assert_eq!(state.biome, Biome::SummerIsles); // 55 % 3 = 1
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = GameState::new("测试".to_string(), 0);
        state.move_cursor(1, 1);
        assert_eq!(state.cursor, (1, 1));

        // 边界测试
        state.move_cursor(-10, -10);
        assert_eq!(state.cursor, (0, 0));
    }
}
