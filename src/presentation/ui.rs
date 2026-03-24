use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::Stylize,
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
                        Constraint::Length(3),
                    ])
                    .split(frame.area());

                render_ascii_header(frame, app, chunks[0], "Rules Management", "View and manage firewall rules");
                rules::render_rules_list(frame, app, chunks[1]);
                rules::render_footer(frame, app, chunks[2]);

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
                        Constraint::Length(3),
                    ])
                    .split(frame.area());

                render_ascii_header(frame, app, chunks[0], "Log Management", "System logs & purging");

                logs::render_logs_list(frame, app, chunks[1]);

                let paragraph = Paragraph::new(" ↑↓: Navigate | r: Refresh | m: Menu | q: exit ")
                    .block(Block::new().borders(Borders::ALL))
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(paragraph, chunks[2]);
            }

            AppMode::QuarantineList => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(12),
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ])
                    .split(frame.area());

                render_ascii_header(frame, app, chunks[0], "IP Quarantine", "Blacklist Management");

                quarantine::render_quarantine_list(frame, app, chunks[1]);

                let footer = if app.show_quarantine_dialog {
                    " Enter: to confirm | Esc: cancel "
                } else {
                    " ↑↓: Navigate | q: New IP address | d: Delete IP address | m: Menu "
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

                let paragraph =
                    Paragraph::new(Line::from(vec![Span::raw(footer), Span::raw(" | "), msg]))
                        .block(Block::new().borders(Borders::ALL))
                        .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(paragraph, chunks[2]);

                if app.show_quarantine_dialog {
                    quarantine::render_quarantine_dialog(frame, app);
                }
            }

            AppMode::RateLimitForm => {
                rate_limit::render_rate_limit_form(frame, app);
            }
        }
    }
}

fn render_ascii_header(frame: &mut Frame, _app: &mut AppState, area: ratatui::layout::Rect, title: &str, description: &str) {
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
        terminal.draw(|f| Ui::render(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
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
