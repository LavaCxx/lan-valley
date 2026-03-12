// LanValley - 终端种田游戏
// 主程序入口

mod game;
mod ui;

use crate::game::{GameMode, GameState, SaveManager, Tool};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取本机 IP 最后一位
    let ip_last_octet = get_local_ip_last_octet();
    
    // 创建存档管理器
    let save_manager = SaveManager::new()?;
    
    // 尝试加载存档，如果失败则创建新游戏
    let mut state = if save_manager.exists() {
        match save_manager.load() {
            Ok(loaded_state) => {
                println!("已加载存档: 第 {} 年 {}季 第 {} 天", 
                    loaded_state.time.year,
                    loaded_state.time.season_name(),
                    loaded_state.time.day
                );
                loaded_state
            }
            Err(e) => {
                println!("存档加载失败，创建新游戏: {}", e);
                GameState::new("玩家".to_string(), ip_last_octet)
            }
        }
    } else {
        println!("首次运行，创建新游戏");
        GameState::new("玩家".to_string(), ip_last_octet)
    };
    
    // 用于自动存档的共享状态
    let state_arc = Arc::new(Mutex::new(state.clone()));
    let save_manager_arc = Arc::new(save_manager);
    
    // 启动自动存档线程
    let auto_save_state = Arc::clone(&state_arc);
    let auto_save_manager = Arc::clone(&save_manager_arc);
    let auto_save_handle = std::thread::spawn(move || {
        let mut last_save = Instant::now();
        loop {
            std::thread::sleep(Duration::from_secs(60));
            {
                let state = auto_save_state.lock().unwrap();
                if last_save.elapsed() >= Duration::from_secs(60) {
                    if let Err(e) = auto_save_manager.save(&state) {
                        eprintln!("自动存档失败: {}", e);
                    } else {
                        // 静默保存成功
                    }
                    last_save = Instant::now();
                }
            }
        }
    });
    
    // 初始化终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // 主循环
    let res = run_game(&mut terminal, &mut state);
    
    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    // 退出时保存
    {
        let save_manager = Arc::try_unwrap(save_manager_arc)
            .map_err(|_| "无法获取存档管理器")?;
        if let Err(e) = save_manager.save(&state) {
            eprintln!("保存失败: {}", e);
        } else {
            println!("游戏已保存");
        }
    }
    
    // 停止自动存档线程
    // auto_save_handle 会随程序退出自动终止
    
    if let Err(err) = res {
        eprintln!("错误: {:?}", err);
    }
    
    Ok(())
}

/// 游戏主循环
fn run_game(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut GameState,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);
    
    loop {
        // 渲染
        terminal.draw(|f| ui::render(f, state))?;
        
        // 处理输入（非阻塞）
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    // 只处理按下事件，忽略重复
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match (key.modifiers, key.code) {
                    // 退出
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) |
                    (KeyModifiers::NONE, KeyCode::Char('q')) => {
                        // 只在正常模式下退出
                        if state.mode == GameMode::Normal {
                            return Ok(());
                        }
                    }
                    
                    // ESC - 关闭菜单
                    (KeyModifiers::NONE, KeyCode::Esc) => {
                        state.close_menu();
                    }
                    
                    // 方向键移动
                    (KeyModifiers::NONE, KeyCode::Up) => {
                        state.move_cursor(0, -1);
                    }
                    (KeyModifiers::NONE, KeyCode::Down) => {
                        state.move_cursor(0, 1);
                    }
                    (KeyModifiers::NONE, KeyCode::Left) => {
                        state.move_cursor(-1, 0);
                    }
                    (KeyModifiers::NONE, KeyCode::Right) => {
                        state.move_cursor(1, 0);
                    }
                    
                    // Enter - 确认菜单选择
                    (KeyModifiers::NONE, KeyCode::Enter) => {
                        state.confirm_menu();
                    }
                    
                    // 以下操作只在正常模式下有效
                    (KeyModifiers::NONE, KeyCode::Char('t')) => {
                        if state.mode == GameMode::Normal {
                            state.till();
                        }
                    }
                    
                    (KeyModifiers::NONE, KeyCode::Char('w')) => {
                        if state.mode == GameMode::Normal {
                            state.water();
                        }
                    }
                    
                    (KeyModifiers::NONE, KeyCode::Char('h')) => {
                        if state.mode == GameMode::Normal {
                            state.harvest();
                        }
                    }
                    
                    (KeyModifiers::NONE, KeyCode::Char('p')) => {
                        if state.mode == GameMode::Normal {
                            state.open_plant_select();
                        }
                    }
                    
                    (KeyModifiers::NONE, KeyCode::Char('s')) => {
                        if state.mode == GameMode::Normal {
                            state.open_shop();
                        }
                    }
                    
                    (KeyModifiers::NONE, KeyCode::Char('i')) => {
                        if state.mode == GameMode::Normal {
                            state.open_inventory();
                        }
                    }
                    
                    // 建筑菜单
                    (KeyModifiers::NONE, KeyCode::Char('b')) => {
                        if state.mode == GameMode::Normal {
                            // 检查是否有可放置的建筑
                            let placeable = state.buildings.get_placeable_buildings();
                            if placeable.is_empty() {
                                state.open_building_shop();
                            } else {
                                // 直接进入放置模式
                                state.start_building_place();
                            }
                        }
                    }
                    
                    // 烹饪菜单
                    (KeyModifiers::NONE, KeyCode::Char('c')) => {
                        if state.mode == GameMode::Normal {
                            state.open_cooking();
                        }
                    }
                    
                    _ => {}
                }
            }
            Event::Mouse(mouse) => {
                let size = terminal.size()?;
                let rect = Rect::new(0, 0, size.width, size.height);
                handle_mouse(state, mouse, rect);
            }
            _ => {}
        }
        }
        
        // 更新游戏状态
        if last_tick.elapsed() >= tick_rate {
            state.update();
            last_tick = Instant::now();
        }
    }
}

/// 处理鼠标事件
fn handle_mouse(state: &mut GameState, mouse: MouseEvent, terminal_size: ratatui::layout::Rect) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let click_x = mouse.column;
            let click_y = mouse.row;

            // 计算布局（与 render.rs 保持一致）
            // 主布局：上半部分（农场+信息+日志）| 下半部分工具栏（3行高）
            let toolbar_height = 3u16;
            let main_height = terminal_size.height.saturating_sub(toolbar_height);

            // 先检查是否点击工具栏区域
            if click_y >= main_height {
                // 工具栏区域：左40字符工具 | 右侧菜单按钮
                let toolbar_inner_y = main_height + 1; // border后

                // 工具选择区（左侧，宽度40）
                if click_x < 40 {
                    let tools = Tool::all();
                    // 每个工具大约占8个字符宽度
                    let toolbar_start_x = 2u16; // border + 1
                    for (i, tool) in tools.iter().enumerate() {
                        let tool_x = toolbar_start_x + (i as u16 * 8);
                        if click_x >= tool_x && click_x < tool_x + 7 {
                            state.select_tool(*tool);
                            return;
                        }
                    }
                } else {
                    // 菜单按钮区（右侧）
                    // 按钮: 🏪 商店 | 🎒 背包 | 🏗️ 建筑 | 🍳 烹饪
                    // 每个按钮大约占10个字符
                    let button_start_x = 42u16; // 40(工具区) + 2(border)
                    let buttons = [
                        ("商店", 's'),
                        ("背包", 'i'),
                        ("建筑", 'b'),
                        ("烹饪", 'c'),
                    ];
                    for (i, (_, key)) in buttons.iter().enumerate() {
                        let btn_x = button_start_x + (i as u16 * 10);
                        if click_x >= btn_x && click_x < btn_x + 9 {
                            match key {
                                's' => state.open_shop(),
                                'i' => state.open_inventory(),
                                'b' => {
                                    let placeable = state.buildings.get_placeable_buildings();
                                    if placeable.is_empty() {
                                        state.open_building_shop();
                                    } else {
                                        state.start_building_place();
                                    }
                                }
                                'c' => state.open_cooking(),
                                _ => {}
                            }
                            return;
                        }
                    }
                }
                return;
            }

            // 农场区域计算（左侧50%）
            let farm_width = (terminal_size.width / 2).saturating_sub(2);
            let farm_height = main_height.saturating_sub(2);

            // 计算网格参数
            let grid_width = state.grid.width as u16 * 3;
            let grid_height = state.grid.height as u16 * 2;

            // 网格起始位置（在农场区域内居中）
            let farm_inner_x = 1u16; // border
            let farm_inner_y = 1u16; // border
            let start_x = farm_inner_x + (farm_width.saturating_sub(grid_width)) / 2;
            let start_y = farm_inner_y + (farm_height.saturating_sub(grid_height)) / 2;

            // 检查是否点击在网格内
            if click_x >= start_x && click_x < start_x + grid_width
               && click_y >= start_y && click_y < start_y + grid_height {
                let grid_x = ((click_x - start_x) / 3) as usize;
                let grid_y = ((click_y - start_y) / 2) as usize;

                if grid_x < state.grid.width && grid_y < state.grid.height {
                    // 如果点击同一位置，执行智能操作
                    if state.cursor == (grid_x, grid_y) {
                        state.smart_action();
                    } else {
                        // 否则移动光标
                        state.set_cursor(grid_x, grid_y);
                    }
                }
            }
        }
        MouseEventKind::Down(MouseButton::Right) => {
            // 右键关闭菜单
            if state.mode != GameMode::Normal {
                state.close_menu();
            }
        }
        _ => {}
    }
}

/// 获取本机 IP 地址最后一位
fn get_local_ip_last_octet() -> u8 {
    use local_ip_address::local_ip;
    
    match local_ip() {
        Ok(ip) => {
            if let std::net::IpAddr::V4(ipv4) = ip {
                let octets = ipv4.octets();
                octets[3]
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}
