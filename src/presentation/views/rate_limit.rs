use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::presentation::app::{AppMode, AppState};
use crate::use_cases::firewall_trait::FirewallRepository;

pub struct RateLimitList {
    pub state: ListState,
    pub rate_limits: Vec<String>,
}

impl RateLimitList {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            rate_limits: Vec::new(),
        }
    }

    pub fn next(&mut self) {
        if self.rate_limits.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.rate_limits.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.rate_limits.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.rate_limits.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

/// Renderiza la lista de reglas de rate limit
pub fn render_rate_limit(frame: &mut Frame, app: &mut AppState, area: Rect) {
    if app.rate_limit_list.rate_limits.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Rate Limit Rules ")
            .title_style(Style::default().fg(Color::Gray));
        let text = Paragraph::new("No rate limit rules found. Press 'a' to add one.")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(text, area);
    } else {
        let selected = app.rate_limit_list.state.selected();
        let items: Vec<ListItem> = app
            .rate_limit_list
            .rate_limits
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                let is_selected = selected == Some(i);
                let prefix = if is_selected {
                    Span::styled(">> ", Style::default().fg(Color::Cyan))
                } else {
                    Span::raw("   ")
                };
                let content = if is_selected {
                    Span::styled(rule, Style::default().fg(Color::Cyan))
                } else {
                    Span::styled(rule, Style::default().fg(Color::Gray))
                };
                ListItem::new(Line::from(vec![prefix, content]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Rate Limit Rules ")
                    .title_style(Style::default().fg(Color::Gray)),
            )
            .highlight_style(Style::default().fg(Color::Cyan));

        frame.render_stateful_widget(list, area, &mut app.rate_limit_list.state);
    }

    if app.show_rate_limit_dialog {
        render_rate_limit_form(frame, app);
    }
}

/// Diálogo flotante para añadir una nueva regla
fn render_rate_limit_form(frame: &mut Frame, app: &AppState) {
    let area = frame.area();
    let dialog_area = Rect {
        x: area.x + (area.width.saturating_sub(50) / 2),
        y: area.y + (area.height.saturating_sub(14) / 2),
        width: 50.min(area.width),
        height: 14.min(area.height),
    };

    frame.render_widget(Clear, dialog_area);

    let block = Block::new()
        .borders(Borders::ALL)
        .title(" Apply Rate Limit ")
        .style(Style::default().bg(Color::Black).fg(Color::Cyan));

    let inner_area = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Port
            Constraint::Length(3), // Protocol
            Constraint::Length(3), // Rate
            Constraint::Length(3), // Unit
        ])
        .split(inner_area);

    // Port
    let port_style = if app.rl_focus == 0 {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(ratatui::style::Modifier::REVERSED)
    } else {
        Style::default().fg(Color::White)
    };
    let port_text = if app.rl_port_input.is_empty() {
        "____".to_string()
    } else {
        app.rl_port_input.clone()
    };
    frame.render_widget(
        Paragraph::new(format!(" {} ", port_text))
            .block(Block::new().borders(Borders::ALL).title(" Port "))
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
        Paragraph::new(format!(" {} ", proto_text))
            .block(Block::new().borders(Borders::ALL).title(" Protocol (Tab) "))
            .style(Style::default().fg(Color::White)),
        chunks[1],
    );

    // Rate
    let rate_style = if app.rl_focus == 1 {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(ratatui::style::Modifier::REVERSED)
    } else {
        Style::default().fg(Color::White)
    };
    frame.render_widget(
        Paragraph::new(format!(" {} ", app.rl_rate_input))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title(" Limit (connections) "),
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
        Paragraph::new(format!(" {} ", unit_text))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title(" Time Unit (Space) "),
            )
            .style(Style::default().fg(Color::White)),
        chunks[3],
    );
}

/// Maneja eventos de la vista de rate limit
pub fn handle_rate_limit_events(key: KeyEvent, app: &mut AppState) {
    use crossterm::event::KeyCode;

    if app.show_rate_limit_dialog {
        match key.code {
            KeyCode::Esc => {
                app.show_rate_limit_dialog = false;
            }
            KeyCode::Up => {
                app.rl_focus = app.rl_focus.saturating_sub(1);
            }
            KeyCode::Down => {
                if app.rl_focus < 1 {
                    app.rl_focus += 1;
                }
            }
            KeyCode::Tab => {
                app.rl_protocol = if app.rl_protocol == "tcp" {
                    "udp".to_string()
                } else {
                    "tcp".to_string()
                };
            }
            KeyCode::Char(' ') => {
                app.rl_unit = if app.rl_unit == "second" {
                    "minute".to_string()
                } else {
                    "second".to_string()
                };
            }
            KeyCode::Enter => {
                app.apply_rate_limit();
                app.show_rate_limit_dialog = false;
                app.refresh_rate_limit_rules();
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
    } else {
        match key.code {
            KeyCode::Up => {
                app.rate_limit_list.previous();
            }
            KeyCode::Down => {
                app.rate_limit_list.next();
            }
            KeyCode::Char('a') => {
                app.show_rate_limit_dialog = true;
                app.rl_port_input.clear();
                app.rl_rate_input = "10".to_string();
                app.rl_focus = 0;
            }
            KeyCode::Char('m') | KeyCode::Esc => {
                app.mode = AppMode::Menu;
            }
            KeyCode::Char('d') => {
                // Eliminar regla seleccionada
                if let Some(selected) = app.rate_limit_list.state.selected() {
                    if selected < app.rate_limit_list.rate_limits.len() {
                        let rule = app.rate_limit_list.rate_limits[selected].clone();

                        match app.repository.delete_rate_limit_rule(&rule) {
                            Ok(_) => {
                                app.refresh_rate_limit_rules();
                                app.message = Some((
                                    false,
                                    "Rate limit rule deleted successfully".to_string(),
                                ));
                            }
                            Err(e) => {
                                app.message = Some((true, format!("Failed to delete rule: {}", e)));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
