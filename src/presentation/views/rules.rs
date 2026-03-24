use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::domain::Rule;
use crate::presentation::app::{AppMode, AppState};

/// Header de la vista de reglas
pub fn render_header(frame: &mut Frame, app: &AppState, area: Rect) {
    let title = format!(" Pathogen - Table: {} ", app.current_table);
    let block = Block::new()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);
}

/// Lista de reglas
pub fn render_rules_list(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let rules = app.get_rules();
    let selected_index = app.selected_index;

    if rules.is_empty() {
        let block = Block::new()
            .title(" Rules ")
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

            let content = format_rule(rule);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::new().title(" Rules ").borders(Borders::ALL));

    frame.render_widget(list, area);
}

/// Formatea una regla para mostrar
pub fn format_rule(rule: &Rule) -> String {
    use crate::domain::Action;

    let action_str = match rule.action {
        Action::Accept => "✓ ACCEPT",
        Action::Drop => "✗ DROP",
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

/// Footer de la vista de reglas
pub fn render_footer(frame: &mut Frame, app: &AppState, area: Rect) {
    use ratatui::text::Span;

    let help_text = if app.show_block_dialog {
        " Enter: Confirm | Esc: Cancel "
    } else {
        " ↑↓: Navigate | b: Block | d: Delete | m: Menu | q: Exit "
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

/// Diálogo para bloquear puerto
pub fn render_block_dialog(frame: &mut Frame, app: &AppState) {
    let area = frame.area();
    let dialog_area = Rect::new(
        area.width.saturating_sub(40) / 2,
        area.height.saturating_sub(10) / 2,
        area.width.saturating_sub(40) / 2 + 40,
        area.height.saturating_sub(10) / 2 + 10,
    );

    frame.render_widget(Clear, dialog_area);

    let block = Block::new()
        .title(" Block Port ")
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

    let instructions = Paragraph::new("Enter the port number (1-65535):")
        .style(Style::default().fg(Color::White));
    frame.render_widget(instructions, chunks[0]);

    let port_text = if app.block_port_input.is_empty() {
        "____".to_string()
    } else {
        app.block_port_input.clone()
    };
    let port_display = Paragraph::new(port_text)
        .block(Block::new().borders(Borders::ALL).title("Port"))
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
                .title("Protocol (Tab to change)"),
        )
        .style(Style::default().fg(Color::White));
    frame.render_widget(protocol_display, chunks[2]);
}

/// Maneja eventos de la vista de reglas
pub fn handle_rules_events(key: KeyEvent, app: &mut AppState) {
    match key.code {
        crossterm::event::KeyCode::Char('m') | crossterm::event::KeyCode::Esc => {
            if !app.show_block_dialog {
                app.mode = AppMode::Menu;
            } else {
                app.show_block_dialog = false;
            }
        }
        crossterm::event::KeyCode::Up => {
            if !app.show_block_dialog && !app.rules.is_empty() {
                app.selected_index = app.selected_index.saturating_sub(1);
            }
        }
        crossterm::event::KeyCode::Down => {
            if !app.show_block_dialog && !app.rules.is_empty() {
                let max = app.rules.len() - 1;
                if app.selected_index < max {
                    app.selected_index += 1;
                }
            }
        }
        crossterm::event::KeyCode::Char('b') => {
            if !app.show_block_dialog {
                app.show_block_dialog = true;
                app.block_port_input.clear();
            }
        }
        crossterm::event::KeyCode::Char('d') => {
            if !app.show_block_dialog {
                app.delete_rule();
            }
        }
        crossterm::event::KeyCode::Enter => {
            if app.show_block_dialog {
                app.block_port();
            }
        }
        crossterm::event::KeyCode::Char(c) => {
            if app.show_block_dialog {
                if c.is_ascii_digit() && app.block_port_input.len() < 5 {
                    app.block_port_input.push(c);
                }
            }
        }
        crossterm::event::KeyCode::Backspace => {
            if app.show_block_dialog {
                app.block_port_input.pop();
            }
        }
        crossterm::event::KeyCode::Tab => {
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
