// 游戏核心模块

pub mod building;
pub mod cooking;
pub mod grid;
pub mod inventory;
pub mod save;
pub mod state;
pub mod types;

// 主要对外接口
pub use save::SaveManager;
pub use state::{GameMode, GameState, MenuItem};

// 常用类型重新导出
pub use types::{
    Biome, BuildingType, Crop, CropStage, CropType, DishType, GameTime, ItemType, Season,
    SoilState, Tile, Weather,
};
