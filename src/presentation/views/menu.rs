use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{List, ListItem, Paragraph}, //Block, Borders,
};

use crate::presentation::app::{AppMode, AppState};
use crate::presentation::ascii_art;
use crossterm::event::KeyEvent;

/// Renderiza el menú principal
pub fn render_menu(frame: &mut Frame, app: &mut AppState) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // ASCII Art
            Constraint::Length(4),  // Descripción
            Constraint::Min(10),    // Menú
        ])
        .margin(2)
        .split(area);

    let art_lines = ascii_art::render_ascii_art();

    let p = Paragraph::new(art_lines)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(p, chunks[0]);

    let desc = Paragraph::new(
        "Secure Firewall TUI Powered by nftables\nRuthless filtering. Absolute network control.",
    )
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(desc, chunks[1]);

    let items = vec![
        " Rule Configuration & Port Management",
        " IP Quarantine (Blacklist)",
        " Traffic Control (Anti-DDoS)",
        " Manage Rate Limit Rules",
        " Log Management & Purging",
        " Terminate Session",
    ];

    let list_items: Vec<ListItem> = items
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if i == app.menu_index {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(ratatui::style::Modifier::REVERSED)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(t).style(style)
        })
        .collect();

    let list = List::new(list_items).highlight_symbol(">> ");

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
        ])
        .split(chunks[2]);

    frame.render_widget(list, horizontal_layout[1]);
}

/// Maneja los eventos del menú
/// Retorna true si el usuario quiere salir
pub fn handle_menu_events(key: KeyEvent, app: &mut AppState) -> bool {
    match key.code {
        crossterm::event::KeyCode::Up => {
            app.menu_index = app.menu_index.saturating_sub(1);
        }
        crossterm::event::KeyCode::Down => {
            if app.menu_index < 5 {
                app.menu_index += 1;
            }
        }
        crossterm::event::KeyCode::Enter => {
            match app.menu_index {
                0 => {
                    app.mode = AppMode::RulesList;
                    app.refresh_rules();
                }
                1 => {
                    app.mode = AppMode::QuarantineList;
                    app.refresh_quarantine();
                }
                2 => {
                    app.mode = AppMode::RateLimitForm;
                }
                3 => {
                    app.mode = AppMode::RateLimitList;
                }
                4 => {
                    app.mode = AppMode::LogsViewer;
                    app.refresh_logs();
                }
                _ => return true, // Salir
            }
        }
        _ => {}
    }
    false
}
