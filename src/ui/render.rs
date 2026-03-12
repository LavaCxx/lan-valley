// UI 渲染

use crate::game::{GameMode, GameState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// 主渲染函数
pub fn render(f: &mut Frame, state: &GameState) {
    // 主布局：左侧农场 | 中间信息 | 右侧日志
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
        ])
        .split(f.area());

    // 渲染农场
    render_farm(f, state, chunks[0]);

    // 渲染信息面板
    render_info(f, state, chunks[1]);

    // 渲染日志
    render_logs(f, state, chunks[2]);
}

/// 渲染农场
fn render_farm(f: &mut Frame, state: &GameState, area: Rect) {
    let block = Block::default()
        .title(format!(
            " 🌾 {} - {} {}年第{}季第{}天 ",
            state.biome.name(),
            state.weather.icon(),
            state.time.year,
            state.time.season_name(),
            state.time.day
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 计算网格起始位置（居中）
    let grid_width = state.grid.width as u16 * 3; // 每个格子3字符宽
    let grid_height = state.grid.height as u16 * 2; // 每个格子2字符高

    let start_x = inner.x + (inner.width.saturating_sub(grid_width)) / 2;
    let start_y = inner.y + (inner.height.saturating_sub(grid_height)) / 2;

    // 渲染每个地块
    for y in 0..state.grid.height {
        for x in 0..state.grid.width {
            let tile = state.grid.get(x, y).unwrap();
            let (icon, style) = get_tile_display(tile, x, y, state);

            let cell_x = start_x + (x as u16 * 3);
            let cell_y = start_y + (y as u16 * 2);

            // 如果是光标位置，使用高亮样式
            let is_cursor = state.cursor == (x, y) && state.mode == GameMode::Normal;
            let final_style = if is_cursor {
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                style
            };

            let span = Span::styled(icon, final_style);
            f.render_widget(Paragraph::new(span), Rect::new(cell_x, cell_y, 3, 1));
        }
    }

    // 渲染建筑
    for building in state.buildings.all() {
        let cell_x = start_x + (building.x as u16 * 3);
        let cell_y = start_y + (building.y as u16 * 2);
        let span = Span::styled(building.icon(), Style::default().fg(Color::Cyan));
        f.render_widget(Paragraph::new(span), Rect::new(cell_x, cell_y, 3, 1));
    }
}

/// 获取地块显示
fn get_tile_display(
    tile: &crate::game::Tile,
    x: usize,
    y: usize,
    state: &GameState,
) -> (String, ratatui::style::Style) {
    use ratatui::style::{Color, Modifier, Style};

    // 检查是否有建筑
    if let Some(building) = state.buildings.get_at(x, y) {
        return (
            building.icon().to_string(),
            Style::default().fg(Color::Cyan),
        );
    }

    // 检查是否有作物
    if let Some(crop) = &tile.crop {
        let (icon, style) = if crop.is_mature() {
            (
                crop.icon().to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (
                crop.icon().to_string(),
                Style::default().fg(Color::Green),
            )
        };

        // 显示浇水状态：已浇水的作物加蓝色背景
        if crop.watered {
            return (icon, Style::default().fg(Color::Cyan).bg(Color::Rgb(30, 60, 90)));
        }
        return (icon, style);
    }

    // 显示土地状态
    match tile.soil {
        crate::game::SoilState::Grass => ("🌿".to_string(), Style::default().fg(Color::DarkGray)),
        crate::game::SoilState::Tilled => (
            "⬜".to_string(),
            Style::default().fg(Color::Rgb(139, 90, 43)),
        ),
        crate::game::SoilState::Watered => ("💧".to_string(), Style::default().fg(Color::Blue)),
    }
}

/// 渲染信息面板
fn render_info(f: &mut Frame, state: &GameState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 状态栏
            Constraint::Length(8), // 操作提示
            Constraint::Min(5),    // 菜单/背包
        ])
        .split(area);

    // 状态栏
    let status = Paragraph::new(vec![Line::from(vec![
        Span::styled("💰 ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{} G", state.inventory.gold),
            Style::default().fg(Color::Yellow),
        ),
    ])])
    .block(Block::default().borders(Borders::ALL).title(" 状态 "));
    f.render_widget(status, chunks[0]);

    // 操作提示
    let help_text = match state.mode {
        GameMode::Normal => vec![
            "方向键: 移动光标",
            "T: 耕地  W: 浇水",
            "P: 种植  H: 收获",
            "S: 商店  I: 背包",
            "B: 建筑  C: 烹饪",
            "Q: 退出",
        ],
        GameMode::Shop => vec!["↑↓: 选择", "Enter: 购买", "ESC: 关闭"],
        GameMode::Inventory => vec!["↑↓: 选择", "Enter: 出售", "ESC: 关闭"],
        GameMode::PlantSelect => vec!["↑↓: 选择", "Enter: 种植", "ESC: 关闭"],
        GameMode::BuildingShop => vec!["↑↓: 选择", "Enter: 购买", "ESC: 关闭"],
        GameMode::BuildingPlace => vec!["方向键: 选择位置", "Enter: 放置", "ESC: 取消"],
        GameMode::Cooking => vec!["↑↓: 选择", "Enter: 烹饪", "ESC: 关闭"],
    };

    let help = Paragraph::new(help_text.iter().map(|s| Line::from(*s)).collect::<Vec<_>>())
        .block(Block::default().borders(Borders::ALL).title(" 操作 "));
    f.render_widget(help, chunks[1]);

    // 菜单
    let menu_items = state.get_menu_items();
    if !menu_items.is_empty() {
        let items: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == state.menu_index {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(item.label.clone()).style(style)
            })
            .collect();

        let title = match state.mode {
            GameMode::Shop => " 🏪 商店 ",
            GameMode::Inventory => " 🎒 背包 ",
            GameMode::PlantSelect => " 🌱 种植 ",
            GameMode::BuildingShop => " 🏗️ 建筑 ",
            GameMode::Cooking => " 🍳 烹饪 ",
            _ => " 菜单 ",
        };

        let menu = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(menu, chunks[2]);
    } else if state.mode == GameMode::Normal {
        // 显示当前地块信息
        if let Some(info) = state.current_tile_info() {
            let info_widget = Paragraph::new(info).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 📍 地块信息 "),
            );
            f.render_widget(info_widget, chunks[2]);
        }
    }
}

/// 渲染日志
fn render_logs(f: &mut Frame, state: &GameState, area: Rect) {
    let logs: Vec<ListItem> = state
        .logs
        .iter()
        .rev()
        .take(10)
        .map(|log| ListItem::new(log.clone()))
        .collect();

    let log_widget =
        List::new(logs).block(Block::default().borders(Borders::ALL).title(" 📜 日志 "));
    f.render_widget(log_widget, area);
}
