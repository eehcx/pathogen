use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame, Terminal,
};
use std::io;
use crossterm::event::{self, KeyCode};

use crate::domain::Rule;
use crate::presentation::app::{AppState, AppMode};

/// UI component for the TUI
pub struct Ui;

impl Ui {
    /// Render the main UI
    pub fn render(frame: &mut Frame, app: &mut AppState) {
        match app.mode {
            AppMode::Menu => Self::render_menu(frame, app),
            AppMode::RulesList => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ])
                    .split(frame.area());

                Self::render_header(frame, app, chunks[0]);
                Self::render_rules_list(frame, app, chunks[1]);
                Self::render_footer(frame, app, chunks[2]);

                if app.show_block_dialog {
                    Self::render_block_dialog(frame, app);
                }
            },
            AppMode::LogsViewer => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ])
                    .split(frame.area());

                let title = " Pathogen Firewall - Registros de Purga (Logs) ";
                let block = Block::new()
                    .title(title)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Red));
                frame.render_widget(block, chunks[0]);

                Self::render_logs_list(frame, app, chunks[1]);

                let paragraph = Paragraph::new(" ↑↓: Navegar | r: Recargar | m: Menú | q: Salir ")
                    .block(Block::new().borders(Borders::ALL))
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(paragraph, chunks[2]);
            }
            AppMode::QuarantineList => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ])
                    .split(frame.area());

                let title = " Pathogen Firewall - Cuarentena (IP Blacklist) ";
                let block = Block::new()
                    .title(title)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Magenta));
                frame.render_widget(block, chunks[0]);

                Self::render_quarantine_list(frame, app, chunks[1]);

                let footer = if app.show_quarantine_dialog {
                    " Enter: Confirmar | Esc: Cancelar "
                } else {
                    " ↑↓: Navegar | q: Añadir IP | d: Eliminar IP | m: Menú "
                };

                let msg = if let Some((is_error, msg)) = &app.message {
                    if *is_error { Span::raw(msg.as_str()).fg(Color::Red) } else { Span::raw(msg.as_str()).fg(Color::Green) }
                } else { Span::raw("") };

                let paragraph = Paragraph::new(Line::from(vec![Span::raw(footer), Span::raw(" | "), msg]))
                    .block(Block::new().borders(Borders::ALL))
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(paragraph, chunks[2]);

                if app.show_quarantine_dialog {
                    Self::render_quarantine_dialog(frame, app);
                }
            },
            AppMode::RateLimitForm => {
                Self::render_rate_limit_form(frame, app);
            }
        }
    }

    fn render_menu(frame: &mut Frame, app: &mut AppState) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // ASCII Art (Biomecánico)
                Constraint::Length(4), // Desc
                Constraint::Min(10)    // Menu
            ])
            .margin(2)
            .split(area);

        let ascii_art = r#"
 ██▓███   ▄▄▄     ▄▄▄█████▓ ██░ ██  ▒█████   ▄████  ▓█████  ███▄    █ 
▓██░  ██▒▒████▄   ▓  ██▒ ▓▒▓██░ ██▒▒██▒  ██▒██▒ ▀█▒ ▓█   ▀  ██ ▀█   █ 
▓██░ ██▓▒▒██  ▀█▄ ▒ ▓██░ ▒░▒██▀▀██░▒██░  ██▒██░▄▄▄░ ▒███   ▓██  ▀█ ██▒
▒██▄█▓▒ ▒░██▄▄▄▄██░ ▓██▓ ░ ░▓█ ░██ ▒██   ██░▓█  ██▓ ▒▓█  ▄ ▓██▒  ▐▌██▒
▒██▒ ░  ░ ▓█   ▓██▒ ▒██▒ ░ ░▓█▒░██▓░ ████▓▒░▒▓███▀▒░▒████▒ ▒██░   ▓██░
▒▓▒░ ░  ░ ▒▒   ▓▒█░ ▒ ░░    ▒ ░░▒░▒░ ▒░▒░▒░ ░▒   ▒  ░ ▒░ ░ ░ ▒░   ▒ ▒ 
░▒ ░       ▒   ▒▒ ░   ░     ▒ ░▒░ ░  ░ ▒ ▒░  ░   ░  ░ ░  ░ ░ ░░   ░ ▒░
░░         ░   ▒    ░       ░  ░░ ░░ ░ ░ ▒ ░ ░   ░    ░       ░   ░ ░ 
               ░  ░         ░  ░  ░    ░ ░       ░    ░  ░          ░ "#;

        let p = Paragraph::new(ascii_art)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan)); // Cian oscuro (estilo Prometheus)
        
        frame.render_widget(p, chunks[0]);

        let desc = Paragraph::new("Secure Firewall TUI Powered by nftables\nPurga el tráfico impuro. Redefine las reglas de supervivencia.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(desc, chunks[1]);

        let items = vec![
            "Listar / Gestionar Reglas (Puertos)",
            "Cuarentena de IPs (Blacklist)",
            "Control de Flujo (Anti-DDoS)",
            "Registros de Purga (Logs)",
            "Salir de la Consola"
        ];

        let list_items: Vec<ListItem> = items.into_iter().enumerate().map(|(i, t)| {
            let style = if i == app.menu_index {
                // Color turquesa/cian sucio (Prometheus Holograms)
                Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::REVERSED)
            } else {
                // Color gris oscuro/verde apagado (LV-223 / Giger aesthetics)
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(t).style(style)
        }).collect();

        let list = List::new(list_items)
            .highlight_symbol(">> ");

        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Percentage(30),
                Constraint::Percentage(35)
            ]).split(chunks[2]);

        frame.render_widget(list, horizontal_layout[1]);
    }

    fn render_header(frame: &mut Frame, app: &AppState, area: Rect) {
        let title = format!(" Pathogen Firewall - Tabla: {} ", app.current_table);
        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(block, area);
    }

    fn render_rules_list(frame: &mut Frame, app: &mut AppState, area: Rect) {
        let rules = app.get_rules();
        let selected_index = app.selected_index;

        if rules.is_empty() {
            let block = Block::new()
                .title(" Reglas ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));

            let text = Paragraph::new("No hay reglas disponibles o error al cargar.")
                .block(block)
                .alignment(Alignment::Center);

            frame.render_widget(text, area);
            return;
        }

        let items: Vec<ListItem> = rules
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(ratatui::style::Modifier::REVERSED)
                } else {
                    Style::default().fg(Color::White)
                };

                let content = Self::format_rule(rule);
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items).block(Block::new().title(" Reglas ").borders(Borders::ALL));

        frame.render_widget(list, area);
    }

    fn format_rule(rule: &Rule) -> String {
        let action_str = match rule.action {
            crate::domain::Action::Accept => "✓ ACCEPT",
            crate::domain::Action::Drop => "✗ DROP",
        };

        let status = if rule.enabled { "ON " } else { "OFF" };
        let src = rule
            .src_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "*".to_string());
        let dst = rule
            .dst_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "*".to_string());

        let comment = rule.comment.as_deref().unwrap_or("");

        if comment.is_empty() {
            format!(
                "{:3} {:10} {:8} {:5} {}:{} {}",
                rule.priority, rule.chain, action_str, rule.protocol, src, dst, status
            )
        } else {
            format!(
                "{:3} {:10} {:8} {:5} {}:{} {} | {}",
                rule.priority, rule.chain, action_str, rule.protocol, src, dst, status, comment
            )
        }
    }

    fn render_logs_list(frame: &mut Frame, app: &mut AppState, area: Rect) {
        if app.logs.is_empty() {
            let block = Block::new()
                .title(" Logs ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray));

            let text = Paragraph::new("No hay registros de purga recientes.")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
            return;
        }

        let items: Vec<ListItem> = app.logs.iter().map(|log| {
            ListItem::new(log.as_str()).style(Style::default().fg(Color::Red))
        }).collect();

        let list = List::new(items)
            .block(Block::new().title(" Logs ").borders(Borders::ALL));

        frame.render_widget(list, area);
    }

    fn render_quarantine_list(frame: &mut Frame, app: &mut AppState, area: Rect) {
        if app.quarantined_ips.is_empty() {
            let block = Block::new().title(" IPs en Cuarentena ").borders(Borders::ALL).style(Style::default().fg(Color::White));
            let text = Paragraph::new("No hay IPs en cuarentena actualmente.").block(block).alignment(Alignment::Center);
            frame.render_widget(text, area);
            return;
        }

        let items: Vec<ListItem> = app.quarantined_ips.iter().enumerate().map(|(i, ip)| {
            let style = if i == app.quarantine_index {
                Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::REVERSED)
            } else {
                Style::default().fg(Color::Magenta)
            };
            ListItem::new(format!("  {}  ", ip)).style(style)
        }).collect();

        let list = List::new(items).block(Block::new().title(" IPs ").borders(Borders::ALL));
        frame.render_widget(list, area);
    }

    fn render_quarantine_dialog(frame: &mut Frame, app: &AppState) {
        let area = frame.area();
        let dialog_area = Rect::new(
            area.width.saturating_sub(40) / 2,
            area.height.saturating_sub(8) / 2,
            area.width.saturating_sub(40) / 2 + 40,
            area.height.saturating_sub(8) / 2 + 8,
        );

        frame.render_widget(Clear, dialog_area);

        let block = Block::new()
            .title(" Añadir IP a Cuarentena ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::Magenta));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Length(3)])
            .split(inner_area);

        frame.render_widget(Paragraph::new("Ingrese la dirección IPv4:").style(Style::default().fg(Color::White)), chunks[0]);

        let ip_text = if app.quarantine_ip_input.is_empty() { "_._._._".to_string() } else { app.quarantine_ip_input.clone() };
        let ip_display = Paragraph::new(ip_text).block(Block::new().borders(Borders::ALL)).style(Style::default().fg(Color::Yellow));
        frame.render_widget(ip_display, chunks[1]);
    }

    fn render_rate_limit_form(frame: &mut Frame, app: &mut AppState) {
        let area = frame.area();
        let dialog_area = Rect::new(
            area.width.saturating_sub(50) / 2,
            area.height.saturating_sub(18) / 2,
            area.width.saturating_sub(50) / 2 + 50,
            area.height.saturating_sub(18) / 2 + 18,
        );

        let block = Block::new()
            .title(" Control de Flujo (Anti-DDoS) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Port
                Constraint::Length(3), // Protocol
                Constraint::Length(3), // Rate
                Constraint::Length(3), // Unit
                Constraint::Length(2), // Msg
                Constraint::Min(2),    // Footer
            ])
            .split(inner_area);

        // Port
        let port_style = if app.rl_focus == 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) };
        let port_text = if app.rl_port_input.is_empty() { "____".to_string() } else { app.rl_port_input.clone() };
        frame.render_widget(Paragraph::new(port_text).block(Block::new().borders(Borders::ALL).title("Puerto")).style(port_style), chunks[0]);

        // Protocol
        let proto_text = if app.rl_protocol == "tcp" { "[TCP] UDP" } else { "TCP [UDP]" };
        frame.render_widget(Paragraph::new(proto_text).block(Block::new().borders(Borders::ALL).title("Protocolo (Tab)")).style(Style::default().fg(Color::White)), chunks[1]);

        // Rate
        let rate_style = if app.rl_focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) };
        frame.render_widget(Paragraph::new(app.rl_rate_input.clone()).block(Block::new().borders(Borders::ALL).title("Límite (conexiones)")).style(rate_style), chunks[2]);

        // Unit
        let unit_text = if app.rl_unit == "second" { "[Segundo] Minuto" } else { "Segundo [Minuto]" };
        frame.render_widget(Paragraph::new(unit_text).block(Block::new().borders(Borders::ALL).title("Unidad de Tiempo (Espacio)")).style(Style::default().fg(Color::White)), chunks[3]);

        // Msg
        if let Some((is_error, msg)) = &app.message {
            let color = if *is_error { Color::Red } else { Color::Green };
            frame.render_widget(Paragraph::new(msg.as_str()).style(Style::default().fg(color)), chunks[4]);
        }

        // Footer
        frame.render_widget(Paragraph::new(" ↑↓: Campo | Enter: Aplicar | m: Menú ").style(Style::default().fg(Color::DarkGray)), chunks[5]);
    }

    fn render_footer(frame: &mut Frame, app: &AppState, area: Rect) {
        let help_text = if app.show_block_dialog {
            " Enter: Confirmar | Esc: Cancelar "
        } else {
            " ↑↓: Navegar | b: Bloquear | d: Eliminar | m: Menú | q: Salir "
        };

        let msg = if let Some((is_error, msg)) = &app.message {
            if *is_error {
                Span::raw(msg.as_str()).fg(Color::Red)
            } else {
                Span::raw(msg.as_str()).fg(Color::Green)
            }
        } else {
            Span::raw("")
        };

        let paragraph = Paragraph::new(Line::from(vec![
            Span::raw(help_text),
            Span::raw(" | "),
            msg,
        ]))
        .block(Block::new().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    fn render_block_dialog(frame: &mut Frame, app: &AppState) {
        let area = frame.area();
        let dialog_area = Rect::new(
            area.width.saturating_sub(40) / 2,
            area.height.saturating_sub(10) / 2,
            area.width.saturating_sub(40) / 2 + 40,
            area.height.saturating_sub(10) / 2 + 10,
        );

        frame.render_widget(Clear, dialog_area); // Clear background behind popup

        let block = Block::new()
            .title(" Bloquear Puerto ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(inner_area);

        let instructions = Paragraph::new("Ingrese el número de puerto (1-65535):")
            .style(Style::default().fg(Color::White));
        frame.render_widget(instructions, chunks[0]);

        let port_text = if app.block_port_input.is_empty() {
            "____".to_string()
        } else {
            app.block_port_input.clone()
        };
        let port_display = Paragraph::new(port_text)
            .block(Block::new().borders(Borders::ALL).title("Puerto"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(port_display, chunks[1]);

        let protocol_text = if app.block_protocol == "tcp" {
            "[TCP] UDP"
        } else {
            "TCP [UDP]"
        };
        let protocol_display = Paragraph::new(protocol_text)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Protocolo (Tab para cambiar)"),
            )
            .style(Style::default().fg(Color::White));
        frame.render_widget(protocol_display, chunks[2]);
    }
}


fn handle_menu_events(key: event::KeyEvent, app: &mut AppState) -> bool {
    match key.code {
        KeyCode::Up => {
            app.menu_index = app.menu_index.saturating_sub(1);
        }
        KeyCode::Down => {
            if app.menu_index < 4 { app.menu_index += 1; }
        }
        KeyCode::Enter => {
            match app.menu_index {
                0 => { app.mode = AppMode::RulesList; app.refresh_rules(); }
                1 => { app.mode = AppMode::QuarantineList; app.refresh_quarantine(); }
                2 => { app.mode = AppMode::RateLimitForm; }
                3 => { app.mode = AppMode::LogsViewer; app.refresh_logs(); }
                _ => { return true; } // Signal to quit
            }
        }
        _ => {}
    }
    false
}

fn handle_logs_events(key: event::KeyEvent, app: &mut AppState) {
    match key.code {
        KeyCode::Char('m') | KeyCode::Esc => {
            app.mode = AppMode::Menu;
        }
        KeyCode::Char('r') => {
            app.refresh_logs();
        }
        _ => {}
    }
}

fn handle_quarantine_events(key: event::KeyEvent, app: &mut AppState) {
    match key.code {
        KeyCode::Char('m') | KeyCode::Esc => {
            if !app.show_quarantine_dialog {
                app.mode = AppMode::Menu;
            } else {
                app.show_quarantine_dialog = false;
            }
        }
        KeyCode::Up => {
            if !app.show_quarantine_dialog && !app.quarantined_ips.is_empty() {
                app.quarantine_index = app.quarantine_index.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if !app.show_quarantine_dialog && !app.quarantined_ips.is_empty() {
                let max = app.quarantined_ips.len() - 1;
                if app.quarantine_index < max {
                    app.quarantine_index += 1;
                }
            }
        }
        KeyCode::Char('q') => {
            if !app.show_quarantine_dialog {
                app.show_quarantine_dialog = true;
                app.quarantine_ip_input.clear();
            }
        }
        KeyCode::Char('d') => {
            if !app.show_quarantine_dialog {
                app.remove_quarantine();
            }
        }
        KeyCode::Enter => {
            if app.show_quarantine_dialog {
                app.quarantine_ip();
            }
        }
        KeyCode::Char(c) => {
            if app.show_quarantine_dialog {
                if (c.is_ascii_digit() || c == '.') && app.quarantine_ip_input.len() < 15 {
                    app.quarantine_ip_input.push(c);
                }
            }
        }
        KeyCode::Backspace => {
            if app.show_quarantine_dialog {
                app.quarantine_ip_input.pop();
            }
        }
        _ => {}
    }
}

fn handle_rate_limit_events(key: event::KeyEvent, app: &mut AppState) {
    match key.code {
        KeyCode::Char('m') | KeyCode::Esc => {
            app.mode = AppMode::Menu;
        }
        KeyCode::Up => {
            app.rl_focus = app.rl_focus.saturating_sub(1);
        }
        KeyCode::Down => {
            if app.rl_focus < 1 { app.rl_focus += 1; }
        }
        KeyCode::Tab => {
            app.rl_protocol = if app.rl_protocol == "tcp" { "udp".to_string() } else { "tcp".to_string() };
        }
        KeyCode::Char(' ') => {
            app.rl_unit = if app.rl_unit == "second" { "minute".to_string() } else { "second".to_string() };
        }
        KeyCode::Enter => {
            app.apply_rate_limit();
        }
        KeyCode::Char(c) => {
            if c.is_ascii_digit() {
                if app.rl_focus == 0 && app.rl_port_input.len() < 5 {
                    app.rl_port_input.push(c);
                } else if app.rl_focus == 1 && app.rl_rate_input.len() < 5 {
                    app.rl_rate_input.push(c);
                }
            }
        }
        KeyCode::Backspace => {
            if app.rl_focus == 0 {
                app.rl_port_input.pop();
            } else if app.rl_focus == 1 {
                app.rl_rate_input.pop();
            }
        }
        _ => {}
    }
}

fn handle_rules_events(key: event::KeyEvent, app: &mut AppState) {
    match key.code {
        KeyCode::Char('m') | KeyCode::Esc => {
            if !app.show_block_dialog {
                app.mode = AppMode::Menu;
            } else {
                app.show_block_dialog = false;
            }
        }
        KeyCode::Up => {
            if !app.show_block_dialog && !app.rules.is_empty() {
                app.selected_index = app.selected_index.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if !app.show_block_dialog && !app.rules.is_empty() {
                let max = app.rules.len() - 1;
                if app.selected_index < max {
                    app.selected_index += 1;
                }
            }
        }
        KeyCode::Char('b') => {
            if !app.show_block_dialog {
                app.show_block_dialog = true;
                app.block_port_input.clear();
            }
        }
        KeyCode::Char('d') => {
            if !app.show_block_dialog {
                app.delete_rule();
            }
        }
        KeyCode::Enter => {
            if app.show_block_dialog {
                app.block_port();
            }
        }
        KeyCode::Char(c) => {
            if app.show_block_dialog {
                if c.is_ascii_digit() && app.block_port_input.len() < 5 {
                    app.block_port_input.push(c);
                }
            }
        }
        KeyCode::Backspace => {
            if app.show_block_dialog {
                app.block_port_input.pop();
            }
        }
        KeyCode::Tab => {
            if app.show_block_dialog {
                app.block_protocol = if app.block_protocol == "tcp" {
                    "udp".to_string()
                } else {
                    "tcp".to_string()
                };
            }
        }
        _ => {}
    }
}

pub fn run_tui() -> Result<(), io::Error> {
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new();

    loop {
        terminal.draw(|f| Ui::render(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                app.clear_message();

                if key.code == KeyCode::Char('q') && app.mode == AppMode::Menu {
                    break;
                }

                match app.mode {
                    AppMode::Menu => {
                        if handle_menu_events(key, &mut app) {
                            break;
                        }
                    },
                    AppMode::LogsViewer => handle_logs_events(key, &mut app),
                    AppMode::QuarantineList => handle_quarantine_events(key, &mut app),
                    AppMode::RateLimitForm => handle_rate_limit_events(key, &mut app),
                    AppMode::RulesList => handle_rules_events(key, &mut app),
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
