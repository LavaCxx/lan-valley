// LanValley - 终端种田游戏
// 主程序入口

mod game;
mod ui;

use crate::game::{GameMode, GameState, SaveManager};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
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
            if let Event::Key(key) = event::read()? {
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
        }
        
        // 更新游戏状态
        if last_tick.elapsed() >= tick_rate {
            state.update();
            last_tick = Instant::now();
        }
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
