// UI 样式定义

use ratatui::style::{Color, Modifier, Style};

/// 样式常量
pub struct Styles;

impl Styles {
    /// 默认样式
    pub fn default() -> Style {
        Style::default()
    }

    /// 光标高亮
    pub fn cursor() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    }

    /// 成熟作物
    pub fn crop_mature() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    /// 生长中作物
    pub fn crop_growing() -> Style {
        Style::default().fg(Color::Green)
    }

    /// 建筑
    pub fn building() -> Style {
        Style::default().fg(Color::Cyan)
    }

    /// 草地
    pub fn soil_grass() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// 耕地
    pub fn soil_tilled() -> Style {
        Style::default().fg(Color::Rgb(139, 90, 43))
    }

    /// 已浇水
    pub fn soil_watered() -> Style {
        Style::default().fg(Color::Blue)
    }

    /// 金币
    pub fn gold() -> Style {
        Style::default().fg(Color::Yellow)
    }

    /// 菜单选中项
    pub fn menu_selected() -> Style {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    }

    /// 菜单普通项
    pub fn menu_normal() -> Style {
        Style::default()
    }

    /// 标题边框
    pub fn border_title() -> Style {
        Style::default().fg(Color::Green)
    }

    /// 日志文本
    pub fn log() -> Style {
        Style::default().fg(Color::Gray)
    }

    /// 晴天
    pub fn weather_sunny() -> Style {
        Style::default().fg(Color::Yellow)
    }

    /// 下雨
    pub fn weather_rainy() -> Style {
        Style::default().fg(Color::Blue)
    }
}
