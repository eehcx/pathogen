use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::io;

use crate::presentation::app::{AppMode, AppState};
use crate::presentation::views::{logs, menu, quarantine, rate_limit, rules};
use crate::presentation::ascii_art;

/// UI component for the TUI
pub struct Ui;

impl Ui {
    /// Render the main UI - solo orquesta las vistas
    pub fn render(frame: &mut Frame, app: &mut AppState) {
        match app.mode {
            AppMode::Menu => menu::render_menu(frame, app),

            AppMode::RulesList => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(12),
                        Constraint::Min(0),
                        Constraint::Length(2),
                    ])
                    .split(frame.area());

                render_ascii_header(frame, app, chunks[0], "Rules Management", "View and manage firewall rules");
                rules::render_rules_list(frame, app, chunks[1]);
                
                let help = if app.show_block_dialog {
                    "[Enter] Confirm  [Esc] Cancel"
                } else {
                    "[↑↓] Navigate  [b] Block Port  [d] Delete Rule  [m] Menu  [q] Exit"
                };
                render_global_footer(frame, app, chunks[2], help);

                if app.show_block_dialog {
                    rules::render_block_dialog(frame, app);
                }
            }

            AppMode::LogsViewer => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(12),
                        Constraint::Min(0),
                        Constraint::Length(2),
                    ])
                    .split(frame.area());

                render_ascii_header(frame, app, chunks[0], "Log Management", "System logs & purging");

                logs::render_logs_list(frame, app, chunks[1]);

                render_global_footer(frame, app, chunks[2], "[↑↓] Navigate  [r] Refresh  [m] Menu  [q] Exit");
            }

            AppMode::QuarantineList => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(12),
                        Constraint::Min(0),
                        Constraint::Length(2),
                    ])
                    .split(frame.area());

                render_ascii_header(frame, app, chunks[0], "IP Quarantine", "Blacklist Management");

                quarantine::render_quarantine_list(frame, app, chunks[1]);

                let help = if app.show_quarantine_dialog {
                    "[Enter] Confirm  [Esc] Cancel"
                } else {
                    "[↑↓] Navigate  [q] Quarantine IP  [d] Delete IP  [m] Menu"
                };
                render_global_footer(frame, app, chunks[2], help);

                if app.show_quarantine_dialog {
                    quarantine::render_quarantine_dialog(frame, app);
                }
            }

            AppMode::RateLimitForm => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(12),
                        Constraint::Min(0),
                        Constraint::Length(2),
                    ])
                    .split(frame.area());

                render_ascii_header(frame, app, chunks[0], "Traffic Control", "Anti-DDoS and Rate Limiting");
                
                rate_limit::render_rate_limit_form(frame, app, chunks[1]);
                
                render_global_footer(frame, app, chunks[2], "[↑↓] Field  [Tab] Protocol  [Space] Unit  [Enter] Apply  [m] Menu");
            }
        }

        if app.rollback_active {
            render_rollback_warning(frame, app);
        }
    }
}

fn render_rollback_warning(frame: &mut Frame, app: &AppState) {
    use ratatui::widgets::Clear;
    let area = frame.area();
    
    let popup_area = ratatui::layout::Rect::new(
        area.width.saturating_sub(60) / 2,
        area.height.saturating_sub(10) / 2,
        60.min(area.width),
        10.min(area.height),
    );

    frame.render_widget(Clear, popup_area);

    let secs_left = app.rollback_deadline
        .map(|d| d.saturating_duration_since(std::time::Instant::now()).as_secs())
        .unwrap_or(0);

    let block = Block::new()
        .title(" WARNING: CRITICAL NETWORK CHANGES ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Red));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let text = vec![
        Line::from(Span::styled("Changes have been applied to the firewall.", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(
            format!("Reverting in {} seconds...", secs_left),
            Style::default().fg(Color::Red).bold()
        )),
        Line::from(""),
        Line::from(Span::styled("[ENTER] Confirm Changes    [ESC] Revert Now", Style::default().fg(Color::Yellow))),
    ];

    let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, inner_area);
}

pub fn render_global_footer(frame: &mut Frame, app: &AppState, area: Rect, help_text: &str) {
    let msg = if let Some((is_error, msg)) = &app.message {
        if *is_error {
            Span::styled(format!(" [ERROR: {}] ", msg), Style::default().fg(Color::Red).bold())
        } else {
            Span::styled(format!(" [OK: {}] ", msg), Style::default().fg(Color::Green).bold())
        }
    } else {
        Span::raw("")
    };

    let footer_text = Line::from(vec![
        Span::styled(help_text, Style::default().fg(Color::DarkGray)),
        Span::raw("   "),
        msg,
    ]);

    let paragraph = Paragraph::new(footer_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::new().borders(Borders::TOP).style(Style::default().fg(Color::Rgb(60, 60, 60))));

    frame.render_widget(paragraph, area);
}

fn render_ascii_header(frame: &mut Frame, _app: &mut AppState, area: Rect, title: &str, description: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Para el arte ASCII (6 líneas)
            Constraint::Length(2),  // Para el título y la línea separadora
            Constraint::Min(0),
        ])
        .split(area);

    // Renderizar arte ASCII
    let ascii_art = ascii_art::render_logo();
    let paragraph = Paragraph::new(ascii_art)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(paragraph, chunks[0]);

    // Título y descripción
    let title_line = Line::from(vec![
        Span::styled(title, Style::default().fg(Color::Gray).bold()),
        Span::styled(" - ", Style::default().fg(Color::DarkGray)),
        Span::styled(description, Style::default().fg(Color::DarkGray)),
    ]);

    let title_paragraph = Paragraph::new(title_line)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::new().borders(Borders::BOTTOM).style(Style::default().fg(Color::Rgb(60, 60, 60)))); // Divider oscuro
    frame.render_widget(title_paragraph, chunks[1]);
}

pub fn run_tui() -> io::Result<()> {
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::{Terminal, backend::CrosstermBackend};

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new();

    loop {
        // Handle rollback timeout
        if app.rollback_active {
            if let Some(deadline) = app.rollback_deadline {
                if std::time::Instant::now() >= deadline {
                    app.cancel_rollback();
                }
            }
        }

        terminal.draw(|f| Ui::render(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if app.rollback_active {
                    match key.code {
                        KeyCode::Enter => {
                            app.confirm_rollback();
                        }
                        KeyCode::Esc => {
                            app.cancel_rollback();
                        }
                        _ => {}
                    }
                    continue;
                }

                app.clear_message();

                if key.code == KeyCode::Char('q') && app.mode == AppMode::Menu {
                    break;
                }

                match app.mode {
                    AppMode::Menu => {
                        if menu::handle_menu_events(key, &mut app) {
                            break;
                        }
                    }
                    AppMode::LogsViewer => logs::handle_logs_events(key, &mut app),
                    AppMode::QuarantineList => quarantine::handle_quarantine_events(key, &mut app),
                    AppMode::RateLimitForm => rate_limit::handle_rate_limit_events(key, &mut app),
                    AppMode::RulesList => rules::handle_rules_events(key, &mut app),
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
