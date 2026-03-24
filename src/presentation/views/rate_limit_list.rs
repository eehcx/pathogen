use crate::presentation::app::{AppState, AppMode};
use crate::use_cases::firewall_trait::FirewallRepository;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub struct RateLimitList {
    pub state: ListState,
    pub rate_limits: Vec<String>, // Lista de reglas de rate limit
}

impl RateLimitList {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            rate_limits: Vec::new(), // Se llenará con datos reales
        }
    }

    pub fn next(&mut self) {
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

pub fn render_rate_limit_list(frame: &mut Frame, app: &AppState, area: Rect) {
    // Crear lista de reglas
    let items: Vec<ListItem> = app
        .rate_limit_list
        .rate_limits
        .iter()
        .enumerate()
        .map(|(i, rule)| {
            let is_selected = app.rate_limit_list.state.selected() == Some(i);
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
        .highlight_style(Style::default().fg(Color::Cyan))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app.rate_limit_list.state.clone());
}

pub fn handle_rate_limit_list_events(key: crossterm::event::KeyEvent, app: &mut AppState) {
    use crossterm::event::KeyCode;
    
    match key.code {
        KeyCode::Up => {
            app.rate_limit_list.previous();
        }
        KeyCode::Down => {
            app.rate_limit_list.next();
        }
        KeyCode::Char('m') | KeyCode::Esc => {
            app.mode = AppMode::Menu;
        }
        KeyCode::Char('d') => {
            // Eliminar regla seleccionada
            if let Some(selected) = app.rate_limit_list.state.selected() {
                if selected < app.rate_limit_list.rate_limits.len() {
                    let rule = app.rate_limit_list.rate_limits[selected].clone();
                    
                    // Intentar eliminar la regla usando el repositorio
                    match app.repository.delete_rate_limit_rule(&rule) {
                        Ok(_) => {
                            // Actualizar la lista localmente
                            app.rate_limit_list.rate_limits.remove(selected);
                            if app.rate_limit_list.rate_limits.is_empty() {
                                app.rate_limit_list.state.select(None);
                            } else if selected >= app.rate_limit_list.rate_limits.len() {
                                app.rate_limit_list.state.select(Some(app.rate_limit_list.rate_limits.len() - 1));
                            }
                            app.message = Some((false, "Rate limit rule deleted successfully".to_string()));
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