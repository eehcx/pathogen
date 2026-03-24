use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    //text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::presentation::app::{AppMode, AppState};
//use crate::presentation::ascii_art::render_logo;

/// Lista de IPs en cuarentena
pub fn render_quarantine_list(frame: &mut Frame, app: &mut AppState, area: Rect) {
    if app.quarantined_ips.is_empty() {
        let block = Block::new()
            .title(" Quarantined IPs ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        let text = Paragraph::new("No IPs are currently in quarantine.")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
        return;
    }

    let items: Vec<ListItem> = app
        .quarantined_ips
        .iter()
        .enumerate()
        .map(|(i, ip)| {
            let style = if i == app.quarantine_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(ratatui::style::Modifier::REVERSED)
            } else {
                Style::default().fg(Color::Magenta)
            };
            ListItem::new(format!("  {}  ", ip)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::new().title(" IPs ").borders(Borders::ALL));
    frame.render_widget(list, area);
}

/// Diálogo para añadir IP
pub fn render_quarantine_dialog(frame: &mut Frame, app: &AppState) {
    let area = frame.area();
    let dialog_area = Rect::new(
        area.width.saturating_sub(40) / 2,
        area.height.saturating_sub(8) / 2,
        area.width.saturating_sub(40) / 2 + 40,
        area.height.saturating_sub(8) / 2 + 8,
    );

    frame.render_widget(Clear, dialog_area);

    let block = Block::new()
        .title(" Add IP to quarantine ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Magenta));

    let inner_area = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)])
        .split(inner_area);

    frame.render_widget(
        Paragraph::new("Enter the IPv4 address:").style(Style::default().fg(Color::White)),
        chunks[0],
    );

    let ip_text = if app.quarantine_ip_input.is_empty() {
        "_._._._".to_string()
    } else {
        app.quarantine_ip_input.clone()
    };
    let ip_display = Paragraph::new(ip_text)
        .block(Block::new().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(ip_display, chunks[1]);
}

/// Maneja eventos de la vista de cuarentena
pub fn handle_quarantine_events(key: KeyEvent, app: &mut AppState) {
    match key.code {
        crossterm::event::KeyCode::Char('m') | crossterm::event::KeyCode::Esc => {
            if !app.show_quarantine_dialog {
                app.mode = AppMode::Menu;
            } else {
                app.show_quarantine_dialog = false;
            }
        }
        crossterm::event::KeyCode::Up => {
            if !app.show_quarantine_dialog && !app.quarantined_ips.is_empty() {
                app.quarantine_index = app.quarantine_index.saturating_sub(1);
            }
        }
        crossterm::event::KeyCode::Down => {
            if !app.show_quarantine_dialog && !app.quarantined_ips.is_empty() {
                let max = app.quarantined_ips.len() - 1;
                if app.quarantine_index < max {
                    app.quarantine_index += 1;
                }
            }
        }
        crossterm::event::KeyCode::Char('q') => {
            if !app.show_quarantine_dialog {
                app.show_quarantine_dialog = true;
                app.quarantine_ip_input.clear();
            }
        }
        crossterm::event::KeyCode::Char('d') => {
            if !app.show_quarantine_dialog {
                app.remove_quarantine();
            }
        }
        crossterm::event::KeyCode::Enter => {
            if app.show_quarantine_dialog {
                app.quarantine_ip();
            }
        }
        crossterm::event::KeyCode::Char(c) => {
            if app.show_quarantine_dialog {
                if (c.is_ascii_digit() || c == '.') && app.quarantine_ip_input.len() < 15 {
                    app.quarantine_ip_input.push(c);
                }
            }
        }
        crossterm::event::KeyCode::Backspace => {
            if app.show_quarantine_dialog {
                app.quarantine_ip_input.pop();
            }
        }
        _ => {}
    }
}
