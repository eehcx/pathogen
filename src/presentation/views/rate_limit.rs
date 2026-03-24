use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    //text::Span,
    widgets::{Block, Borders, Paragraph},
};

use crate::presentation::app::{AppMode, AppState};

/// Formulario de rate limit
pub fn render_rate_limit_form(frame: &mut Frame, app: &mut AppState) {
    let area = frame.area();
    let dialog_area = Rect::new(
        area.width.saturating_sub(50) / 2,
        area.height.saturating_sub(18) / 2,
        area.width.saturating_sub(50) / 2 + 50,
        area.height.saturating_sub(18) / 2 + 18,
    );

    let block = Block::new()
        .title(" Traffic Control (Anti-DDoS) ")
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
    let port_style = if app.rl_focus == 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let port_text = if app.rl_port_input.is_empty() {
        "____".to_string()
    } else {
        app.rl_port_input.clone()
    };
    frame.render_widget(
        Paragraph::new(port_text)
            .block(Block::new().borders(Borders::ALL).title("Port"))
            .style(port_style),
        chunks[0],
    );

    // Protocol
    let proto_text = if app.rl_protocol == "tcp" {
        "[TCP] UDP"
    } else {
        "TCP [UDP]"
    };
    frame.render_widget(
        Paragraph::new(proto_text)
            .block(Block::new().borders(Borders::ALL).title("Protocol (Tab)"))
            .style(Style::default().fg(Color::White)),
        chunks[1],
    );

    // Rate
    let rate_style = if app.rl_focus == 1 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    frame.render_widget(
        Paragraph::new(app.rl_rate_input.clone())
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Limit (connections)"),
            )
            .style(rate_style),
        chunks[2],
    );

    // Unit
    let unit_text = if app.rl_unit == "second" {
        "[Second] Minute"
    } else {
        "Second [Minute]"
    };
    frame.render_widget(
        Paragraph::new(unit_text)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Time Unit (Space)"),
            )
            .style(Style::default().fg(Color::White)),
        chunks[3],
    );

    // Msg
    if let Some((is_error, msg)) = &app.message {
        let color = if *is_error { Color::Red } else { Color::Green };
        frame.render_widget(
            Paragraph::new(msg.as_str()).style(Style::default().fg(color)),
            chunks[4],
        );
    }

    // Footer
    frame.render_widget(
        Paragraph::new(" ↑↓: Field | Enter: Apply | m: Menu ")
            .style(Style::default().fg(Color::DarkGray)),
        chunks[5],
    );
}

/// Maneja eventos del formulario de rate limit
pub fn handle_rate_limit_events(key: KeyEvent, app: &mut AppState) {
    match key.code {
        crossterm::event::KeyCode::Char('m') | crossterm::event::KeyCode::Esc => {
            app.mode = AppMode::Menu;
        }
        crossterm::event::KeyCode::Up => {
            app.rl_focus = app.rl_focus.saturating_sub(1);
        }
        crossterm::event::KeyCode::Down => {
            if app.rl_focus < 1 {
                app.rl_focus += 1;
            }
        }
        crossterm::event::KeyCode::Tab => {
            app.rl_protocol = if app.rl_protocol == "tcp" {
                "udp".to_string()
            } else {
                "tcp".to_string()
            };
        }
        crossterm::event::KeyCode::Char(' ') => {
            app.rl_unit = if app.rl_unit == "second" {
                "minute".to_string()
            } else {
                "second".to_string()
            };
        }
        crossterm::event::KeyCode::Enter => {
            app.apply_rate_limit();
        }
        crossterm::event::KeyCode::Char(c) => {
            if c.is_ascii_digit() {
                if app.rl_focus == 0 && app.rl_port_input.len() < 5 {
                    app.rl_port_input.push(c);
                } else if app.rl_focus == 1 && app.rl_rate_input.len() < 5 {
                    app.rl_rate_input.push(c);
                }
            }
        }
        crossterm::event::KeyCode::Backspace => {
            if app.rl_focus == 0 {
                app.rl_port_input.pop();
            } else if app.rl_focus == 1 {
                app.rl_rate_input.pop();
            }
        }
        _ => {}
    }
}
