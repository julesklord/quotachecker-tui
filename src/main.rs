use std::{
    io,
    time::{Duration, Instant},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{backend::CrosstermBackend, Terminal};

mod agent;
mod config;
mod ui;
mod tests;

use agent::{AgentId, AgentScanner, AgentState};
use config::AppConfig;
use ui::RenderContext;

type ScanResult = (Vec<AgentState>, Vec<String>);

struct App {
    config: AppConfig,
    agents: Vec<AgentState>,
    active_tab: usize,
    selected_agent_idx: usize,
    selected_setting_idx: usize,
    
    // Modal State
    show_budget_modal: bool,
    editing_limit: u32,
    editing_value: String,
    
    logs: Vec<String>,
    tick_count: u64,
    should_quit: bool,
    last_refresh: Instant,
}

impl App {
    fn new() -> Self {
        let config = AppConfig::load();
        let agents = AgentScanner::scan(&config);
        
        let mut logs = Vec::new();
        logs.push("QuotaChecker-CLI TUI initialized successfully.".to_string());
        
        for agent in &agents {
            if agent.executable_path.is_some() {
                let auth_desc = if agent.is_authenticated {
                    format!("Connected ({})", agent.auth_info)
                } else {
                    "Disconnected".to_string()
                };
                logs.push(format!(
                    "Detected agent {} (version: {}) - {}.",
                    agent.name,
                    agent.version.as_deref().unwrap_or("unknown"),
                    auth_desc
                ));
            }
        }
        
        Self {
            config,
            agents,
            active_tab: 0,
            selected_agent_idx: 0,
            selected_setting_idx: 0,
            show_budget_modal: false,
            editing_limit: 0,
            editing_value: String::new(),
            logs,
            tick_count: 0,
            should_quit: false,
            last_refresh: Instant::now(),
        }
    }

    fn refresh_states(&mut self) {
        self.agents = AgentScanner::scan(&self.config);
        self.last_refresh = Instant::now();
        self.add_log("Manual synchronization completed.");
    }

    fn add_log(&mut self, msg: impl Into<String>) {
        self.logs.push(msg.into());
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    fn open_budget_modal(&mut self) {
        let agent = &self.agents[self.selected_agent_idx];
        if agent.executable_path.is_none() {
            self.add_log(format!("Cannot edit quota for uninstalled agent: {}", agent.name));
            return;
        }

        self.editing_limit = agent.quota_limit;
        self.editing_value = self.editing_limit.to_string();
        self.show_budget_modal = true;
        self.add_log(format!("Editing quota limits for {}.", agent.name));
    }

    fn save_budget_modal(&mut self) {
        if let Ok(val) = self.editing_value.parse::<u32>() {
            self.editing_limit = val;
            let agent_id = self.agents[self.selected_agent_idx].id;
            match agent_id {
                AgentId::Codex => {
                    self.config.codex_quota.limit = val;
                }
                AgentId::OpenCode => {
                    self.config.opencode_quota.limit = val;
                }
                AgentId::GeminiCli => {
                    self.config.gemini_quota.limit = val;
                }
                AgentId::Agy => {
                    self.config.agy_quota.limit = val;
                }
                AgentId::Zed => {
                    self.config.zed_quota.limit = val;
                }
            }

            if let Ok(()) = self.config.save() {
                self.show_budget_modal = false;
                self.add_log(format!(
                    "Quota limit for {} saved successfully ({} requests).",
                    self.agents[self.selected_agent_idx].name,
                    val
                ));
                
                // Update UI state instantly
                self.agents[self.selected_agent_idx].quota_limit = val;
                let used = self.agents[self.selected_agent_idx].quota_used;
                self.agents[self.selected_agent_idx].quota_remaining = val.saturating_sub(used);
            } else {
                self.add_log("Error saving configuration to disk.");
            }
        } else {
            self.add_log("Invalid numeric format.");
        }
    }

    fn handle_setting_change(&mut self, next: bool) {
        use config::TuiTheme;
        match self.selected_setting_idx {
            0 => {
                // Cycle Theme
                let current = self.config.theme;
                let new_theme = match current {
                    TuiTheme::Cyan => if next { TuiTheme::Purple } else { TuiTheme::Monochrome },
                    TuiTheme::Purple => if next { TuiTheme::Emerald } else { TuiTheme::Cyan },
                    TuiTheme::Emerald => if next { TuiTheme::Amber } else { TuiTheme::Purple },
                    TuiTheme::Amber => if next { TuiTheme::Monochrome } else { TuiTheme::Emerald },
                    TuiTheme::Monochrome => if next { TuiTheme::Cyan } else { TuiTheme::Amber },
                };
                self.config.theme = new_theme;
                self.add_log(format!("Changed TUI theme to {:?}", new_theme));
            }
            1 => {
                // Cycle Sync Rate: 1000ms -> 2000ms -> 5000ms -> 10000ms
                let current = self.config.refresh_rate_ms;
                let new_rate = match current {
                    1000 => if next { 2000 } else { 10000 },
                    2000 => if next { 5000 } else { 1000 },
                    5000 => if next { 10000 } else { 2000 },
                    _ => if next { 1000 } else { 5000 },
                };
                self.config.refresh_rate_ms = new_rate;
                self.add_log(format!("Changed sync rate to {}ms", new_rate));
            }
            2 => {
                // Cycle Soft Limit: 70% -> 80% -> 90%
                let current = self.config.soft_limit_percent as u32;
                let new_pct = match current {
                    70 => if next { 80 } else { 90 },
                    80 => if next { 90 } else { 70 },
                    _ => if next { 70 } else { 80 },
                };
                self.config.soft_limit_percent = new_pct as f64;
                self.add_log(format!("Changed soft limit warning to {}%", new_pct));
            }
            3 => {
                // Cycle Hard Limit: 90% -> 100% -> 110%
                let current = self.config.hard_limit_percent as u32;
                let new_pct = match current {
                    90 => if next { 100 } else { 110 },
                    100 => if next { 110 } else { 90 },
                    _ => if next { 90 } else { 100 },
                };
                self.config.hard_limit_percent = new_pct as f64;
                self.add_log(format!("Changed hard limit threshold to {}%", new_pct));
            }
            _ => {}
        }
        let _ = self.config.save();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup Terminal and Panic Hooks
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // 2. Initialize App
    let mut app = App::new();
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    // 3. Setup Async Background Scan Thread
    let (tx, rx): (Sender<ScanResult>, Receiver<ScanResult>) = mpsc::channel();
    
    thread::spawn(move || {
        loop {
            // Load latest config from disk dynamically
            let latest_config = AppConfig::load();
            let updated_agents = AgentScanner::scan(&latest_config);
            
            if tx.send((updated_agents, Vec::new())).is_err() {
                break;
            }
            
            thread::sleep(Duration::from_millis(latest_config.refresh_rate_ms));
        }
    });

    // 4. Event Loop
    while !app.should_quit {
        while let Ok((updated_agents, _)) = rx.try_recv() {
            app.agents = updated_agents;
            app.last_refresh = Instant::now();
        }

        // Render UI
        terminal.draw(|f| {
            let ctx = RenderContext {
                active_tab: app.active_tab,
                selected_agent_idx: app.selected_agent_idx,
                selected_setting_idx: app.selected_setting_idx,
                show_budget_modal: app.show_budget_modal,
                editing_value: &app.editing_value,
                logs: &app.logs,
                tick_count: app.tick_count,
                agents: &app.agents,
                config: &app.config,
            };
            ui::draw_ui(f, &ctx);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.show_budget_modal {
                    // Modal Input Event Handler
                    match key.code {
                        KeyCode::Esc => {
                            app.show_budget_modal = false;
                            app.add_log("Cancelled.");
                        }
                        KeyCode::Enter => {
                            app.save_budget_modal();
                        }
                        KeyCode::Backspace => {
                            app.editing_value.pop();
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            app.editing_value.push(c);
                        }
                        _ => {}
                    }
                } else {
                    // Main Screen Event Handler
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        // Change Tabs
                        KeyCode::Tab => {
                            app.active_tab = (app.active_tab + 1) % 5;
                        }
                        KeyCode::Char('1') => app.active_tab = 0,
                        KeyCode::Char('2') => app.active_tab = 1,
                        KeyCode::Char('3') => app.active_tab = 2,
                        KeyCode::Char('4') => app.active_tab = 3,
                        KeyCode::Char('5') => app.active_tab = 4,
                        
                        KeyCode::Left => {
                            if app.active_tab == 4 {
                                app.handle_setting_change(false);
                            } else if app.active_tab > 0 {
                                app.active_tab -= 1;
                            } else {
                                app.active_tab = 4;
                            }
                        }
                        KeyCode::Right => {
                            if app.active_tab == 4 {
                                app.handle_setting_change(true);
                            } else {
                                app.active_tab = (app.active_tab + 1) % 5;
                            }
                        }
                        
                        // Select Agent list or settings list
                        KeyCode::Up => {
                            if app.active_tab == 1 || app.active_tab == 3 {
                                if app.selected_agent_idx > 0 {
                                    app.selected_agent_idx -= 1;
                                } else {
                                    app.selected_agent_idx = app.agents.len() - 1;
                                }
                            } else if app.active_tab == 4 {
                                if app.selected_setting_idx > 0 {
                                    app.selected_setting_idx -= 1;
                                } else {
                                    app.selected_setting_idx = 4;
                                }
                            }
                        }
                        KeyCode::Down => {
                            if app.active_tab == 1 || app.active_tab == 3 {
                                app.selected_agent_idx = (app.selected_agent_idx + 1) % app.agents.len();
                            } else if app.active_tab == 4 {
                                app.selected_setting_idx = (app.selected_setting_idx + 1) % 5;
                            }
                        }
                        
                        // Open Editor
                        KeyCode::Char('s') => {
                            if app.active_tab != 4 {
                                app.open_budget_modal();
                            }
                        }
                        
                        // Force Refresh
                        KeyCode::Char('r') => {
                            app.refresh_states();
                        }
                        
                        KeyCode::Char('e') => {
                            if app.active_tab == 4 {
                                let _ = disable_raw_mode();
                                let mut stdout = io::stdout();
                                let _ = execute!(stdout, LeaveAlternateScreen);
                                
                                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                                if let Some(path) = AppConfig::config_path() {
                                    let _ = std::process::Command::new(editor)
                                        .arg(&path)
                                        .status();
                                    
                                    app.config = AppConfig::load();
                                    app.add_log("Configuration reloaded from disk after manual edit.");
                                }
                                
                                let _ = enable_raw_mode();
                                let mut stdout = io::stdout();
                                let _ = execute!(stdout, EnterAlternateScreen);
                                let _ = terminal.clear();
                            }
                        }
                        
                        KeyCode::Enter if app.active_tab == 4 => {
                            if app.selected_setting_idx == 4 {
                                let _ = disable_raw_mode();
                                let mut stdout = io::stdout();
                                let _ = execute!(stdout, LeaveAlternateScreen);
                                
                                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                                if let Some(path) = AppConfig::config_path() {
                                    let _ = std::process::Command::new(editor)
                                        .arg(&path)
                                        .status();
                                    
                                    app.config = AppConfig::load();
                                    app.add_log("Configuration reloaded from disk after manual edit.");
                                }
                                
                                let _ = enable_raw_mode();
                                let mut stdout = io::stdout();
                                let _ = execute!(stdout, EnterAlternateScreen);
                                let _ = terminal.clear();
                            } else {
                                app.handle_setting_change(true);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick_count += 1;
            last_tick = Instant::now();
        }
    }

    // 5. Restore Terminal Settings on Exit
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    println!("QuotaChecker-CLI dashboard closed safely.");
    Ok(())
}
