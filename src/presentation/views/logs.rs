use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::presentation::app::{AppMode, AppState};

/// Lista de logs
pub fn render_logs_list(frame: &mut Frame, app: &mut AppState, area: Rect) {
    if app.logs.is_empty() {
        let block = Block::new()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray));

        let text = Paragraph::new("No recent records found.")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(text, area);
        return;
    }

    let items: Vec<ListItem> = app
        .logs
        .iter()
        .map(|log| ListItem::new(format!("   {}", log)).style(Style::default().fg(Color::Cyan)))
        .collect();

    let list = List::new(items).block(Block::new().borders(Borders::ALL).style(Style::default().fg(Color::DarkGray)));

    frame.render_widget(list, area);
}

/// Maneja eventos de la vista de logs
pub fn handle_logs_events(key: KeyEvent, app: &mut AppState) {
    match key.code {
        crossterm::event::KeyCode::Char('m') | crossterm::event::KeyCode::Esc => {
            app.mode = AppMode::Menu;
        }
        crossterm::event::KeyCode::Char('r') => {
            app.refresh_logs();
        }
        _ => {}
    }
}
