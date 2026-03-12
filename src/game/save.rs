// 存档系统

use super::state::GameState;
use anyhow::Result;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// 存档管理器
pub struct SaveManager {
    save_path: PathBuf,
}

impl SaveManager {
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "LanValley", "LanValley")
            .ok_or_else(|| anyhow::anyhow!("无法确定存档目录"))?;

        let data_dir = project_dirs.data_dir();
        fs::create_dir_all(data_dir)?;

        let save_path = data_dir.join("save.json");

        Ok(Self { save_path })
    }

    /// 检查存档是否存在
    pub fn exists(&self) -> bool {
        self.save_path.exists()
    }

    /// 保存游戏
    pub fn save(&self, state: &GameState) -> Result<()> {
        let json = serde_json::to_string_pretty(state)?;

        // 原子写入：先写临时文件，再重命名
        let temp_path = self.save_path.with_extension("tmp");
        fs::write(&temp_path, json)?;
        fs::rename(&temp_path, &self.save_path)?;

        Ok(())
    }

    /// 加载游戏
    pub fn load(&self) -> Result<GameState> {
        let json = fs::read_to_string(&self.save_path)?;
        let state: GameState = serde_json::from_str(&json)?;
        Ok(state)
    }

    /// 删除存档
    #[allow(dead_code)]
    pub fn delete(&self) -> Result<()> {
        if self.save_path.exists() {
            fs::remove_file(&self.save_path)?;
        }
        Ok(())
    }
}
