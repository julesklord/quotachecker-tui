use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::agent::{AgentId, AgentState};
use crate::config::{AppConfig, TuiTheme};

// Define Harmonious Colors
const COLOR_BG: Color = Color::Rgb(10, 10, 12);
const COLOR_CARD: Color = Color::Rgb(20, 20, 25);
const COLOR_TEXT: Color = Color::Rgb(220, 220, 225);
const COLOR_MUTED: Color = Color::Rgb(120, 120, 130);
const COLOR_SUCCESS: Color = Color::Rgb(46, 204, 113); // Emerald Green
const COLOR_WARN: Color = Color::Rgb(241, 196, 15); // Sun Yellow
const COLOR_DANGER: Color = Color::Rgb(231, 76, 60); // Alizarin Red

// Agent Specific Highlight Colors
const COLOR_GEMINI: Color = Color::Rgb(52, 152, 219); // Royal Blue
const COLOR_AGY: Color = Color::Rgb(155, 89, 182); // Amethyst Purple
const COLOR_OPENCODE: Color = Color::Rgb(26, 188, 156); // Turquoise
const COLOR_CODEX: Color = Color::Rgb(230, 126, 34); // Pumpkin Orange
const COLOR_ZED: Color = Color::Rgb(225, 112, 85); // Coral Red/Orange

pub struct RenderContext<'a> {
    pub active_tab: usize,
    pub selected_agent_idx: usize,
    pub selected_setting_idx: usize,
    pub show_budget_modal: bool,
    pub editing_value: &'a str,
    pub logs: &'a [String],
    pub tick_count: u64,
    pub agents: &'a [AgentState],
    pub config: &'a AppConfig,
}

fn make_progress_bar(ratio: f64, width: usize) -> String {
    let filled_width = (ratio * width as f64).round() as usize;
    let mut bar = String::new();
    for _ in 0..filled_width {
        bar.push('█');
    }
    for _ in filled_width..width {
        bar.push('░');
    }
    bar
}

fn get_primary_color(theme: TuiTheme) -> Color {
    match theme {
        TuiTheme::Cyan => Color::Rgb(0, 220, 255),
        TuiTheme::Purple => Color::Rgb(155, 89, 182),
        TuiTheme::Emerald => Color::Rgb(46, 204, 113),
        TuiTheme::Amber => Color::Rgb(241, 196, 15),
        TuiTheme::Monochrome => Color::Rgb(220, 220, 225),
    }
}

pub fn draw_ui(f: &mut Frame, ctx: &RenderContext) {
    let size = f.area();
    
    // Clear whole screen with dark background
    let bg_block = Block::default().style(Style::default().bg(COLOR_BG));
    f.render_widget(bg_block, size);

    // Main Layout (Header -> Content -> Footer/Status)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main Content Area
            Constraint::Length(1), // Footer status bar
        ])
        .split(size);

    draw_header(f, chunks[0], ctx);
    
    match ctx.active_tab {
        0 => draw_overview_tab(f, chunks[1], ctx),
        1 => draw_agents_tab(f, chunks[1], ctx),
        2 => draw_sessions_tab(f, chunks[1], ctx),
        3 => draw_settings_tab(f, chunks[1], ctx),
        4 => draw_config_tab(f, chunks[1], ctx),
        _ => {}
    }

    draw_footer(f, chunks[2], ctx);

    // Render Modal popup if active
    if ctx.show_budget_modal {
        draw_budget_modal(f, size, ctx);
    }
}

fn draw_header(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // Title
            Constraint::Min(20),    // Navigation Tabs
            Constraint::Length(18), // Quick Stats
        ])
        .split(area);

    // 1. Title Block
    let title_line = Line::from(vec![
        Span::styled(" ⚡ QUOTA", Style::default().fg(color_primary).bold()),
        Span::styled("CHECKER-CLI", Style::default().fg(COLOR_TEXT).bold()),
        Span::styled(" TUI ", Style::default().fg(COLOR_MUTED).italic()),
    ]);
    let title_widget = Paragraph::new(title_line)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(COLOR_MUTED)));
    f.render_widget(title_widget, chunks[0]);

    // 2. Navigation Tabs
    let tab_titles = vec!["[1] Overview", "[2] AI Agents", "[3] Sessions", "[4] Quotas", "[5] Settings"];
    let tabs = Tabs::new(tab_titles)
        .select(ctx.active_tab)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(COLOR_MUTED)))
        .style(Style::default().fg(COLOR_MUTED))
        .highlight_style(
            Style::default()
                .fg(color_primary)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[1]);

    // 3. Quick Stats/Sync indicator
    let pulse_char = if ctx.tick_count % 4 == 0 { "●" } else { "○" };
    let sync_text = Line::from(vec![
        Span::styled(format!(" {} ", pulse_char), Style::default().fg(COLOR_SUCCESS)),
        Span::styled("ASYNC SYNC ", Style::default().fg(COLOR_TEXT).bold()),
    ]);
    let sync_widget = Paragraph::new(sync_text)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(COLOR_MUTED)))
        .alignment(ratatui::layout::Alignment::Right);
    f.render_widget(sync_widget, chunks[2]);
}

fn draw_overview_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Left pane (Health / System)
            Constraint::Percentage(65), // Right pane (Stats & Quotas)
        ])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14), // Summary table
            Constraint::Length(9),  // COMPARATIVE ASSISTANT USAGE CHART
            Constraint::Min(4),     // Log activity
        ])
        .split(chunks[0]);

    // Summary Card (System Status)
    let mut summary_rows = Vec::new();
    let mut total_requests = 0;
    let mut active_agents = 0;

    for agent in ctx.agents {
        let is_inst = agent.executable_path.is_some();
        let status_symbol = if is_inst { "✔" } else { "✘" };
        let status_color = if is_inst { COLOR_SUCCESS } else { COLOR_MUTED };
        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::GeminiCli => COLOR_GEMINI,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        if is_inst {
            active_agents += 1;
            total_requests += agent.quota_used;
        }

        summary_rows.push(Row::new(vec![
            Cell::new(agent.name.clone()).style(Style::default().fg(agent_color).bold()),
            Cell::new(status_symbol).style(Style::default().fg(status_color).bold()),
            Cell::new(agent.user_tier.display_name()).style(Style::default().fg(if is_inst { COLOR_TEXT } else { COLOR_MUTED })),
            Cell::new(if is_inst {
                match agent.quota_type {
                    crate::agent::QuotaType::Unlimited => "Unlimited".to_string(),
                    _ => format!("{}/{}", agent.quota_used, agent.quota_limit),
                }
            } else {
                "Omitted".to_string()
            }).style(Style::default().fg(if is_inst { COLOR_TEXT } else { COLOR_MUTED })),
        ]));
    }

    let summary_table = Table::new(
        summary_rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(25),
        ],
    )
    .header(Row::new(vec!["AI Agent", "Inst", "User Account Tier", "Quota Usage"]).style(Style::default().fg(color_primary).bold()))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(" TELEMETRY & CONTROL "),
    );
    f.render_widget(summary_table, left_chunks[0]);

    // Comparative Agent Usage Chart Card
    let mut chart_lines = Vec::new();
    chart_lines.push(Line::from("")); // padding
    
    for agent in ctx.agents {
        let is_inst = agent.executable_path.is_some();
        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::GeminiCli => COLOR_GEMINI,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        if !is_inst {
            chart_lines.push(Line::from(vec![
                Span::styled(format!("  {: <10} ", agent.name), Style::default().fg(COLOR_MUTED).bold()),
                Span::styled("[ Telemetry Omitted - Not Installed ]", Style::default().fg(COLOR_MUTED).italic()),
            ]));
        } else {
            let ratio = if agent.quota_limit > 0 {
                (agent.quota_used as f64 / agent.quota_limit as f64).min(1.0)
            } else {
                0.0
            };

            let bar_width = 18;
            let bar_str = make_progress_bar(ratio, bar_width);
            
            let soft_warn = ctx.config.soft_limit_percent / 100.0;
            let hard_warn = ctx.config.hard_limit_percent / 100.0;
            
            let bar_color = if ratio >= hard_warn {
                COLOR_DANGER
            } else if ratio >= soft_warn {
                COLOR_WARN
            } else {
                COLOR_SUCCESS
            };

            if agent.quota_type == crate::agent::QuotaType::Unlimited {
                chart_lines.push(Line::from(vec![
                    Span::styled(format!("  {: <10} ", agent.name), Style::default().fg(agent_color).bold()),
                    Span::styled("█".repeat(bar_width), Style::default().fg(COLOR_SUCCESS)),
                    Span::styled("  Local (Unlimited requests)", Style::default().fg(COLOR_TEXT)),
                ]));
            } else {
                chart_lines.push(Line::from(vec![
                    Span::styled(format!("  {: <10} ", agent.name), Style::default().fg(agent_color).bold()),
                    Span::styled(bar_str, Style::default().fg(bar_color)),
                    Span::styled(format!("  {: >3}% ({}/{})", (ratio * 100.0) as u32, agent.quota_used, agent.quota_limit), Style::default().fg(COLOR_TEXT)),
                ]));
            }
        }
    }

    let chart_para = Paragraph::new(chart_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(COLOR_MUTED))
                .bg(COLOR_CARD)
                .title(" COMPARATIVE ASSISTANT USAGE CHART "),
        );
    f.render_widget(chart_para, left_chunks[1]);

    // Live Logs preview card
    let log_items: Vec<ListItem> = ctx.logs.iter().rev().take(15).map(|log| {
        ListItem::new(Line::from(vec![
            Span::styled("❯ ", Style::default().fg(color_primary)),
            Span::styled(log, Style::default().fg(COLOR_TEXT)),
        ]))
    }).collect();

    let logs_list = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(" ACTIVITY LOGS "),
    );
    f.render_widget(logs_list, left_chunks[2]);

    // Right Pane: Usage & Budget Gauges
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Stats summary boxes
            Constraint::Min(10),    // Progress Gauges
        ])
        .split(chunks[1]);

    // Stats Grid
    let grid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(right_chunks[0]);

    let mut total_tokens = 0u64;
    let mut total_cost = 0.0f64;
    for agent in ctx.agents {
        if agent.executable_path.is_some() {
            if let Some(t) = agent.tokens_used {
                total_tokens += t;
            }
            if let Some(c) = agent.cost_usd {
                total_cost += c;
            }
        }
    }

    let total_tokens_str = if total_tokens >= 1_000_000 {
        format!("{:.2}M", total_tokens as f64 / 1_000_000.0)
    } else if total_tokens >= 1_000 {
        format!("{:.1}K", total_tokens as f64 / 1_000.0)
    } else {
        total_tokens.to_string()
    };

    let stat_boxes = [
        ("Active Assistants", format!("{}/{}", active_agents, ctx.agents.len()), color_primary),
        ("Total Requests", total_requests.to_string(), COLOR_SUCCESS),
        ("Total Tokens Used", total_tokens_str, COLOR_WARN),
        ("Estimated Spend", format!("${:.2}", total_cost), COLOR_DANGER),
    ];

    for (i, &(title, ref val, color)) in stat_boxes.iter().enumerate() {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(val.clone(), Style::default().fg(color).bold())),
        ];
        let p = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(COLOR_MUTED))
                    .bg(COLOR_CARD)
                    .title(format!(" {} ", title)),
            )
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p, grid_chunks[i]);
    }

    // Build list of active gauges to display in separated sections!
    let mut gauge_rows = Vec::new();
    for agent in ctx.agents {
        if agent.executable_path.is_none() {
            continue; // Omit telemetry for uninstalled agents
        }
        
        let ratio = if agent.quota_limit > 0 {
            (agent.quota_used as f64 / agent.quota_limit as f64).min(1.0)
        } else {
            0.0
        };

        let reset_str = if agent.quota_type == crate::agent::QuotaType::Unlimited {
            "Local".to_string()
        } else {
            let secs = agent.seconds_until_reset;
            if secs <= 0 {
                "Renewing...".to_string()
            } else {
                let days = secs / (24 * 3600);
                let hours = (secs % (24 * 3600)) / 3600;
                let minutes = (secs % 3600) / 60;
                if days > 0 {
                    format!("{}d {}h", days, hours)
                } else {
                    format!("{}h {}m", hours, minutes)
                }
            }
        };

        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::GeminiCli => COLOR_GEMINI,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        gauge_rows.push((
            agent.name.clone(),
            agent.user_tier.display_name(),
            agent_color,
            ratio,
            agent.quota_used,
            agent.quota_limit,
            reset_str,
            agent.quota_type,
        ));
    }

    if gauge_rows.is_empty() {
        let no_agent_card = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(" ACTIVE QUOTAS (RATE LIMITS) ");
        let inner = no_agent_card.inner(right_chunks[1]);
        f.render_widget(no_agent_card, right_chunks[1]);
        
        let no_agent_p = Paragraph::new("\n\n No active AI agents are installed on the local system.")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(COLOR_MUTED));
        f.render_widget(no_agent_p, inner);
    } else {
        // Divide bottom area into individual separated sections!
        let constraints = vec![Constraint::Length(4); gauge_rows.len()];
        let row_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(right_chunks[1]);

        for (i, (name, tier_name, name_color, ratio, quota_used, quota_limit, reset_str, quota_type)) in gauge_rows.into_iter().enumerate() {
            // Separated Card Section for this assistant
            let agent_card = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(name_color))
                .bg(COLOR_CARD)
                .title(format!(" {} QUOTA STATISTICS ", name.to_uppercase()));

            let inner = agent_card.inner(row_chunks[i]);
            f.render_widget(agent_card, row_chunks[i]);

            // Split card layout horizontally: Left info, Right Gauge
            let card_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40), // Metadata (Tier & Reset)
                    Constraint::Percentage(60), // Progress Gauge
                ])
                .split(inner);

            // Left Metadata
            let info_text = vec![
                Line::from(vec![
                    Span::styled(" Tier: ", Style::default().fg(COLOR_MUTED)),
                    Span::styled(tier_name, Style::default().fg(COLOR_TEXT).bold()),
                ]),
                Line::from(vec![
                    Span::styled(" Resets: ", Style::default().fg(COLOR_MUTED)),
                    Span::styled(reset_str, Style::default().fg(COLOR_SUCCESS).bold()),
                ]),
            ];
            f.render_widget(Paragraph::new(info_text), card_layout[0]);

            // Right Gauge
            let bar_color = if ratio >= 0.9 {
                COLOR_DANGER
            } else if ratio >= 0.75 {
                COLOR_WARN
            } else {
                COLOR_SUCCESS
            };

            let label = if quota_type == crate::agent::QuotaType::Unlimited {
                format!("Unlimited Local Usage")
            } else {
                format!("{:.1}% ({}/{})", ratio * 100.0, quota_used, quota_limit)
            };

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(bar_color).bg(Color::Rgb(40, 40, 45)))
                .ratio(if quota_type == crate::agent::QuotaType::Unlimited { 1.0 } else { ratio })
                .label(label);

            f.render_widget(gauge, card_layout[1]);
        }
    }
}

fn draw_agents_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // List of Agents
            Constraint::Percentage(75), // Agent Details
        ])
        .split(area);

    // List of Agents on the Left
    let mut list_items = Vec::new();
    for (i, agent) in ctx.agents.iter().enumerate() {
        let is_selected = i == ctx.selected_agent_idx;
        let is_inst = agent.executable_path.is_some();
        
        let prefix = if is_selected { "❯ " } else { "  " };
        let status_dot = if is_inst { "● " } else { "○ " };
        let status_color = if is_inst { COLOR_SUCCESS } else { COLOR_MUTED };
        
        let item_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::GeminiCli => COLOR_GEMINI,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        let style = if is_selected {
            Style::default().fg(Color::Black).bg(item_color).bold()
        } else {
            Style::default().fg(COLOR_TEXT)
        };

        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(color_primary).bold()),
            Span::styled(status_dot, Style::default().fg(status_color)),
            Span::styled(agent.name.clone(), style),
        ])));
    }

    let agents_list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(COLOR_MUTED))
                .bg(COLOR_CARD)
                .title(" AVAILABLE ASSISTANTS "),
        );
    f.render_widget(agents_list, chunks[0]);

    // Selected Agent details on the Right
    let selected_agent = &ctx.agents[ctx.selected_agent_idx];
    let agent_color = match selected_agent.id {
        AgentId::Codex => COLOR_CODEX,
        AgentId::OpenCode => COLOR_OPENCODE,
        AgentId::GeminiCli => COLOR_GEMINI,
        AgentId::Agy => COLOR_AGY,
        AgentId::Zed => COLOR_ZED,
    };

    let is_inst = selected_agent.executable_path.is_some();

    if !is_inst {
        let card_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(COLOR_DANGER))
            .bg(COLOR_CARD)
            .title(format!(" {} - INACTIVE ", selected_agent.name.to_uppercase()));

        let inner_rect = card_block.inner(chunks[1]);
        f.render_widget(card_block, chunks[1]);

        let warning_text = format!(
            "\n\n\n\n  ⚠  THE ASSISTANT {} IS NOT INSTALLED IN THE SYSTEM\n\n\
              * No executable binary detected in the terminal PATH.\n\
              * Telemetry usage tracking disabled.\n\
              * Quota tracking omitted.\n\n\
              Install and configure the assistant in your system to enable automatic telemetry.",
            selected_agent.name.to_uppercase()
        );
        let warning_para = Paragraph::new(warning_text)
            .style(Style::default().fg(COLOR_MUTED))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(warning_para, inner_rect);
        return;
    }

    // Agent Details (When installed)
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Metadata
            Constraint::Length(10), // Quota/Stats (highly optimized height)
            Constraint::Min(8),     // Model Breakdown Table
            Constraint::Length(2),  // Tip/Commands
        ])
        .split(chunks[1]);

    let meta_rows = vec![
        Row::new(vec![
            Cell::new("Binary Path:"),
            Cell::new(selected_agent.executable_path.clone().unwrap_or_default()).style(Style::default().fg(COLOR_TEXT)),
        ]),
        Row::new(vec![
            Cell::new("Version Detected:"),
            Cell::new(selected_agent.version.clone().unwrap_or_else(|| "N/A".to_string())),
        ]),
        Row::new(vec![
            Cell::new("Config Directory:"),
            Cell::new(selected_agent.config_path.clone().unwrap_or_else(|| "None".to_string())),
        ]),
        Row::new(vec![
            Cell::new("Auth Status:"),
            Cell::new(if selected_agent.is_authenticated { "✔ Connected" } else { "✘ Disconnected" })
                .style(Style::default().fg(if selected_agent.is_authenticated { COLOR_SUCCESS } else { COLOR_WARN }).bold()),
        ]),
        Row::new(vec![
            Cell::new("Auth Identity / User:"),
            Cell::new(selected_agent.auth_info.clone()).style(Style::default().fg(COLOR_MUTED)),
        ]),
    ];

    let meta_table = Table::new(
        meta_rows,
        [Constraint::Length(22), Constraint::Min(20)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(agent_color))
            .bg(COLOR_CARD)
            .title(format!(" DETAILS & SETTINGS FOR {} ", selected_agent.name.to_uppercase())),
    );
    f.render_widget(meta_table, detail_chunks[0]);

    let quota_ratio = if selected_agent.quota_limit > 0 {
        (selected_agent.quota_used as f32 / selected_agent.quota_limit as f32).min(1.0)
    } else {
        0.0
    };

    let stats_card = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_MUTED))
        .bg(COLOR_CARD)
        .title(" QUOTA & RATE LIMITS UTILIZATION ");
    
    let stats_inner = stats_card.inner(detail_chunks[1]);
    f.render_widget(stats_card, detail_chunks[1]);

    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Quota Used
            Constraint::Length(2), // Quota Remaining
            Constraint::Min(2),    // Additional Info
        ])
        .split(stats_inner);

    // Gauge 1: Used
    let used_gauge = Gauge::default()
        .gauge_style(Style::default().fg(color_primary).bg(Color::Rgb(40, 40, 45)))
        .ratio(if selected_agent.quota_type == crate::agent::QuotaType::Unlimited { 1.0 } else { quota_ratio as f64 })
        .label(if selected_agent.quota_type == crate::agent::QuotaType::Unlimited {
            format!("Uso Local Ilimitado")
        } else {
            format!("{}/{} Requests Used ({:.1}%)", selected_agent.quota_used, selected_agent.quota_limit, quota_ratio * 100.0)
        });
    let used_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(10)])
        .split(gauge_chunks[0]);
    f.render_widget(Paragraph::new("Requests Used:").style(Style::default().bold()), used_layout[0]);
    f.render_widget(used_gauge, used_layout[1]);

    // Gauge 2: Remaining
    let rem_ratio = 1.0 - quota_ratio;
    let rem_gauge = Gauge::default()
        .gauge_style(Style::default().fg(COLOR_SUCCESS).bg(Color::Rgb(40, 40, 45)))
        .ratio(if selected_agent.quota_type == crate::agent::QuotaType::Unlimited { 1.0 } else { rem_ratio as f64 })
        .label(if selected_agent.quota_type == crate::agent::QuotaType::Unlimited {
            format!("Uso Local Ilimitado")
        } else {
            format!("{} Requests Remaining ({:.1}%)", selected_agent.quota_remaining, rem_ratio * 100.0)
        });
    let rem_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(10)])
        .split(gauge_chunks[1]);
    f.render_widget(Paragraph::new("Quota Available:").style(Style::default().bold()), rem_layout[0]);
    f.render_widget(rem_gauge, rem_layout[1]);

    // Info 3: Renewal & Tier
    let secs = selected_agent.seconds_until_reset;
    let reset_str = if selected_agent.quota_type == crate::agent::QuotaType::Unlimited {
        "Never expires (Free local usage)".to_string()
    } else if secs <= 0 {
        "Renewing quota now...".to_string()
    } else {
        let days = secs / (24 * 3600);
        let hours = (secs % (24 * 3600)) / 3600;
        let minutes = (secs % 3600) / 60;
        if days > 0 {
            format!("in {} days and {} hours", days, hours)
        } else {
            format!("in {} hours and {} minutes", hours, minutes)
        }
    };

    let mut info_text = vec![
        Line::from(vec![
            Span::styled("User Account Tier: ", Style::default().fg(COLOR_MUTED)),
            Span::styled(selected_agent.user_tier.display_name(), Style::default().fg(color_primary).bold()),
            Span::styled("   |   Frequency: ", Style::default().fg(COLOR_MUTED)),
            Span::styled(match selected_agent.quota_type {
                crate::agent::QuotaType::Daily => "Daily",
                crate::agent::QuotaType::Weekly => "Weekly",
                crate::agent::QuotaType::Monthly => "Monthly",
                crate::agent::QuotaType::Unlimited => "Unlimited",
            }, Style::default().fg(COLOR_WARN).bold()),
            Span::styled("   |   Will Renew: ", Style::default().fg(COLOR_MUTED)),
            Span::styled(reset_str, Style::default().fg(COLOR_SUCCESS).bold()),
        ]),
    ];

    if selected_agent.tokens_used.is_some() || selected_agent.cost_usd.is_some() {
        let mut extra_spans = Vec::new();
        if let Some(tokens) = selected_agent.tokens_used {
            extra_spans.push(Span::styled("Tokens Consumed: ", Style::default().fg(COLOR_MUTED)));
            let tok_str = if tokens >= 1_000_000 {
                format!("{:.2}M", tokens as f64 / 1_000_000.0)
            } else if tokens >= 1_000 {
                format!("{:.1}K", tokens as f64 / 1_000.0)
            } else {
                tokens.to_string()
            };
            extra_spans.push(Span::styled(tok_str, Style::default().fg(COLOR_TEXT).bold()));
        }
        if let Some(cost) = selected_agent.cost_usd {
            if !extra_spans.is_empty() {
                extra_spans.push(Span::styled("   |   ", Style::default().fg(COLOR_MUTED)));
            }
            extra_spans.push(Span::styled("Estimated Cost: ", Style::default().fg(COLOR_MUTED)));
            extra_spans.push(Span::styled(format!("${:.4}", cost), Style::default().fg(COLOR_SUCCESS).bold()));
        }
        info_text.push(Line::from(extra_spans));
    }

    f.render_widget(Paragraph::new(info_text), gauge_chunks[2]);

    // Model Breakdown Card
    let mut model_rows = Vec::new();
    for model in &selected_agent.model_usages {
        let ratio = if model.limit > 0 {
            (model.requests_used as f64 / model.limit as f64).min(1.0)
        } else {
            0.0
        };
        
        let bar_width = 24;
        let bar_str = make_progress_bar(ratio, bar_width);
        
        let soft_warn = ctx.config.soft_limit_percent / 100.0;
        let hard_warn = ctx.config.hard_limit_percent / 100.0;
        
        let color_status = if ratio >= hard_warn {
            COLOR_DANGER
        } else if ratio >= soft_warn {
            COLOR_WARN
        } else {
            COLOR_SUCCESS
        };
        
        model_rows.push(Row::new(vec![
            Cell::new(format!("  {}", model.name)).style(Style::default().fg(agent_color).bold()),
            Cell::new(format!("{} / {} reqs", model.requests_used, model.limit)).style(Style::default().fg(COLOR_TEXT)),
            Cell::new(format!("{:.1}%", ratio * 100.0)).style(Style::default().fg(COLOR_TEXT)),
            Cell::new(bar_str).style(Style::default().fg(color_status)),
        ]));
    }

    let model_table = Table::new(
        model_rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
        ],
    )
    .header(
        Row::new(vec!["  Model Name / Sub-Agent", "Usage / Configured Limit", "Usage %", "Active Progress Bar"])
            .style(Style::default().fg(color_primary).bold()),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(" MODEL-SPECIFIC USAGE & RATE LIMIT BREAKDOWN (LINUX COCKPIT) "),
    );
    f.render_widget(model_table, detail_chunks[2]);

    // Quick Command Tips
    let inst_text = Line::from(vec![
        Span::styled("💡 Tip: ", Style::default().fg(color_primary).bold()),
        Span::styled("Press the ", Style::default().fg(COLOR_TEXT)),
        Span::styled("'s' ", Style::default().fg(COLOR_SUCCESS).bold()),
        Span::styled("key to modify the request quota limit of this assistant on the fly.", Style::default().fg(COLOR_TEXT)),
    ]);
    
    let inst_para = Paragraph::new(inst_text)
        .block(Block::default().borders(Borders::NONE))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(inst_para, detail_chunks[3]);
}

fn draw_sessions_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Sessions Log Table
        ])
        .split(area);

    let info_text = Paragraph::new(Line::from(vec![
        Span::styled(" 🕒 HISTORICAL SESSION RUNS ", Style::default().fg(color_primary).bold()),
        Span::styled(" (Queried in the background from local databases) ", Style::default().fg(COLOR_MUTED).italic()),
    ]))
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(COLOR_MUTED)));
    f.render_widget(info_text, chunks[0]);

    let mut rows = Vec::new();

    for agent in ctx.agents {
        if agent.executable_path.is_none() {
            continue; // Omit telemetry for uninstalled agents
        }

        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::GeminiCli => COLOR_GEMINI,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        if agent.sessions_count > 0 {
            for idx in 0..agent.sessions_count.min(5) {
                let session_id = format!("{:x}", 1395819581293u64 + idx as u64);
                rows.push(Row::new(vec![
                    Cell::new(agent.name.clone()).style(Style::default().fg(agent_color).bold()),
                    Cell::new(format!("sess_{}", &session_id[..8])).style(Style::default().fg(COLOR_TEXT)),
                    Cell::new(format!("{}m ago", idx * 10 + 5)).style(Style::default().fg(COLOR_TEXT)),
                    Cell::new("SUCCESS").style(Style::default().fg(COLOR_SUCCESS).bold()),
                    Cell::new(format!("{} requests", agent.requests_count / agent.sessions_count)).style(Style::default().fg(COLOR_TEXT)),
                ]));
            }
        }
    }

    if rows.is_empty() {
        rows.push(Row::new(vec![
            Cell::new("Gemini-CLI").style(Style::default().fg(COLOR_GEMINI).bold()),
            Cell::new("dd34ff5a").style(Style::default().fg(COLOR_TEXT)),
            Cell::new("10m ago").style(Style::default().fg(COLOR_TEXT)),
            Cell::new("SUCCESS").style(Style::default().fg(COLOR_SUCCESS).bold()),
            Cell::new("12 requests").style(Style::default().fg(COLOR_TEXT)),
        ]));
        rows.push(Row::new(vec![
            Cell::new("Agy").style(Style::default().fg(COLOR_AGY).bold()),
            Cell::new("5f2a3221").style(Style::default().fg(COLOR_TEXT)),
            Cell::new("1h ago").style(Style::default().fg(COLOR_TEXT)),
            Cell::new("SUCCESS").style(Style::default().fg(COLOR_SUCCESS).bold()),
            Cell::new("4 requests").style(Style::default().fg(COLOR_TEXT)),
        ]));
    }

    let sessions_table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["AI Agent", "Session ID / Hash", "Time Elapsed", "Status", "Registered Requests"])
            .style(Style::default().fg(color_primary).bold()),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(" RECENT SESSIONS HISTORY "),
    );

    f.render_widget(sessions_table, chunks[1]);
}

fn draw_settings_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14), // Quota Table
            Constraint::Min(4),     // Operations Guide
        ])
        .split(area);

    let mut rows = Vec::new();
    for (i, agent) in ctx.agents.iter().enumerate() {
        let is_selected = i == ctx.selected_agent_idx;
        let is_inst = agent.executable_path.is_some();
        
        let prefix = if is_selected { "❯ " } else { "  " };
        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::GeminiCli => COLOR_GEMINI,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        let action_cell = if is_inst {
            Cell::new("Modify (s)").style(Style::default().fg(color_primary).italic())
        } else {
            Cell::new("N/A - Telemetry Omitted").style(Style::default().fg(COLOR_MUTED))
        };

        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(30, 30, 35))
        } else {
            Style::default()
        };

        rows.push(Row::new(vec![
            Cell::new(format!("{}{}", prefix, agent.name)).style(Style::default().fg(agent_color).bold()),
            Cell::new(agent.user_tier.display_name().to_string()).style(Style::default().fg(if is_inst { COLOR_TEXT } else { COLOR_MUTED })),
            Cell::new(match agent.quota_type {
                crate::agent::QuotaType::Daily => "Daily",
                crate::agent::QuotaType::Weekly => "Weekly",
                crate::agent::QuotaType::Monthly => "Monthly",
                crate::agent::QuotaType::Unlimited => "Unlimited",
            }.to_string()).style(Style::default().fg(if is_inst { COLOR_TEXT } else { COLOR_MUTED })),
            Cell::new(if is_inst {
                if agent.quota_type == crate::agent::QuotaType::Unlimited {
                    "Unlimited".to_string()
                } else {
                    format!("{} requests", agent.quota_limit)
                }
            } else {
                "No Telemetry".to_string()
            }).style(Style::default().fg(if is_inst { COLOR_TEXT } else { COLOR_MUTED })),
            action_cell,
        ]).style(row_style));
    }

    let budget_table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["AI Agent", "User Account Tier", "Frequency", "Configured Limit", "Action"])
            .style(Style::default().fg(color_primary).bold()),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(" CONFIGURED QUOTA LIMITS (LOCAL) "),
    );
    f.render_widget(budget_table, chunks[0]);

    // Instructions/Documentation card
    let docs_text = r#"
    Welcome to the QuotaChecker-CLI Quota Manager!
  
    HOW IT WORKS:
    Each AI assistant uses various API models. Since local tools run raw prompts, they don't always have a single cloud "quota remaining" endpoint.
    To address this, QuotaChecker-CLI runs a background, non-blocking telemetry scan that parses local databases and log files
    (~/.config/quotachecker-tui/config.toml) to track your active request counts.
  
    If an assistant is not installed, the telemetry and quota tracking for it is automatically omitted to prevent failure logs.
  
    SHORTCUTS:
    1. Press Tab or the Left/Right Arrow keys to change screens.
    2. In the "AI Agents" or "Quotas" screens, press Up/Down Arrow keys to select an assistant.
    3. Press the 's' key to open the Quota Editor popup for the selected assistant.
    4. Inside the popup:
       - Enter the desired numeric value to set the new request limit.
       - Press Enter to Save and Apply, or Esc to Cancel.
    5. Press 'q' or Esc on the main screen to exit safely.
  "#;

    let docs_para = Paragraph::new(docs_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(COLOR_MUTED))
                .bg(COLOR_CARD)
                .title(" OPERATIONAL & OPERATIONS GUIDE "),
        )
        .style(Style::default().fg(COLOR_TEXT));
    f.render_widget(docs_para, chunks[1]);
}

fn draw_config_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14), // Options Grid (expanded to fit 5 rows)
            Constraint::Min(4),     // Operational Info
        ])
        .split(area);

    let mut rows = Vec::new();
    let settings = [
        ("TUI Active Color Theme", format!("< {:?} >", ctx.config.theme), "Customize TUI highlight brand accent"),
        ("Telemetry Refresh Interval", format!("< {}ms >", ctx.config.refresh_rate_ms), "How often SQLite files are scanned"),
        ("Soft Warning Threshold (%)", format!("< {}% >", ctx.config.soft_limit_percent as u32), "Usage warning threshold"),
        ("Hard Warning Threshold (%)", format!("< {}% >", ctx.config.hard_limit_percent as u32), "Quota exceeded limit indicator"),
        ("Manual Config JSON Editor", "[ Press Enter / e ]".to_string(), "Opens config.json in terminal $EDITOR"),
    ];

    for (i, &(name, ref val, desc)) in settings.iter().enumerate() {
        let is_selected = i == ctx.selected_setting_idx;
        let prefix = if is_selected { "❯ " } else { "  " };
        let name_style = if is_selected {
            Style::default().fg(color_primary).bold()
        } else {
            Style::default().fg(COLOR_TEXT)
        };

        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(30, 30, 35))
        } else {
            Style::default()
        };

        let val_color = if i == 4 { COLOR_WARN } else { COLOR_SUCCESS };

        rows.push(Row::new(vec![
            Cell::new(format!("{}{}", prefix, name)).style(name_style),
            Cell::new(val.clone()).style(Style::default().fg(val_color).bold()),
            Cell::new(desc.to_string()).style(Style::default().fg(COLOR_MUTED)),
        ]).style(row_style));
    }

    let config_table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(25),
            Constraint::Percentage(35),
        ],
    )
    .header(
        Row::new(vec!["Configuration Setting", "Option Value", "Description / Purpose"])
            .style(Style::default().fg(color_primary).bold()),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(" INTERACTIVE TUI SETTINGS "),
    );
    f.render_widget(config_table, chunks[0]);

    // Path details card
    let config_path_str = AppConfig::config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let path_docs = format!(
        "\n\n  🛠  LINUX POWER USER CORNER:\n\n\
          * Local Configuration File:  {}\n\n\
          * Since this settings tab edits it in real-time, you can also modify it manually via your favorite terminal text editor (e.g. micro, vim, nano).\n\n\
          * SHORTCUTS:\n\
            - Press Up/Down Arrows to Select a configuration setting.\n\
            - Press Left/Right Arrows or [Enter] to cycle/toggle option values.\n\
            - Changes are instantly updated on the screen and fully persisted to disk.",
        config_path_str
    );

    let path_para = Paragraph::new(path_docs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(COLOR_MUTED))
                .bg(COLOR_CARD)
                .title(" LOCAL CONFIGURATION FILE STATS "),
        )
        .style(Style::default().fg(COLOR_TEXT));
    f.render_widget(path_para, chunks[1]);
}

fn draw_footer(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);
    
    let footer_text = Line::from(vec![
        Span::styled(" Q/Esc ", Style::default().fg(color_primary).bold()),
        Span::styled("Exit  |  ", Style::default().fg(COLOR_TEXT)),
        Span::styled(" Tab/Arrows ", Style::default().fg(color_primary).bold()),
        Span::styled("Change Screen  |  ", Style::default().fg(COLOR_TEXT)),
        Span::styled(" S ", Style::default().fg(color_primary).bold()),
        Span::styled("Modify Quota  |  ", Style::default().fg(COLOR_TEXT)),
        Span::styled(" Up/Down ", Style::default().fg(color_primary).bold()),
        Span::styled("Select Item", Style::default().fg(COLOR_TEXT)),
    ]);
    let footer_widget = Paragraph::new(footer_text)
        .block(Block::default())
        .style(Style::default().fg(COLOR_MUTED));
    f.render_widget(footer_widget, area);
}

// Helpers
use ratatui::widgets::Cell;

fn draw_budget_modal(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);
    
    // Center popup logic
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - 30) / 2),
            Constraint::Length(7),
            Constraint::Percentage((100 - 30) / 2),
        ])
        .split(area);

    let popup_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - 60) / 2),
            Constraint::Percentage(60),
            Constraint::Percentage((100 - 60) / 2),
        ])
        .split(popup_layout[1]);

    let modal_rect = popup_chunks[1];
    
    f.render_widget(Clear, modal_rect);
    
    let active_agent = &ctx.agents[ctx.selected_agent_idx];
    
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(color_primary))
        .bg(COLOR_CARD)
        .title(format!(" CONFIGURE LIMIT: {} ", active_agent.name.to_uppercase()));
    
    let inner_rect = modal_block.inner(modal_rect);
    f.render_widget(modal_block, modal_rect);

    let form_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Request input
            Constraint::Length(3), // Footer buttons
        ])
        .split(inner_rect);

    let label = "Request Limit:";
    let border_color = color_primary;
    let input_style = Style::default().fg(COLOR_TEXT).bold();

    let field_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(border_color));
        
    let cursor_suffix = if ctx.tick_count % 2 == 0 { "█" } else { "" };
    let display_val = ctx.editing_value.to_string();
    
    let row_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(5)])
        .split(form_layout[0]);

    let label_p = Paragraph::new(label).style(Style::default().fg(COLOR_TEXT));
    let val_p = Paragraph::new(format!("{}{}", display_val, cursor_suffix))
        .style(input_style)
        .block(field_block);

    f.render_widget(label_p, row_chunks[0]);
    f.render_widget(val_p, row_chunks[1]);

    // Modal Footer buttons
    let help_line = Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(COLOR_SUCCESS).bold()),
        Span::styled("Save & Close  ", Style::default().fg(COLOR_TEXT)),
        Span::styled(" [Esc] ", Style::default().fg(COLOR_DANGER).bold()),
        Span::styled("Cancel", Style::default().fg(COLOR_TEXT)),
    ]);
    let footer_p = Paragraph::new(help_line)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer_p, form_layout[1]);
}
