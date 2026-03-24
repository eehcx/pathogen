use crate::domain::{PortRequest, QuarantineRequest, RateLimitRequest, Rule};
use crate::infrastructure::CliFirewallRepository;
use crate::presentation::views::rate_limit::RateLimitList;
use crate::use_cases::FirewallRepository;
use ratatui::widgets::ListState;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum AppMode {
    Menu,
    RulesList,
    LogsViewer,
    QuarantineList,
    RateLimit,
}

/// AppState represents the application state
pub struct AppState {
    pub rate_limit_list: RateLimitList,
    pub repository: CliFirewallRepository,
    pub mode: AppMode,
    pub rules: Vec<Rule>,
    pub rules_state: ListState, // State for scrollable rules list
    pub logs: Vec<String>,
    pub quarantined_ips: Vec<String>,
    pub selected_index: usize,
    pub menu_index: usize,
    pub log_scroll: usize,
    pub quarantine_index: usize,

    // UI Dialogs State
    pub show_block_dialog: bool,
    pub block_port_input: String,
    pub block_protocol: String,

    pub show_quarantine_dialog: bool,
    pub quarantine_ip_input: String,

    pub show_rate_limit_dialog: bool,

    // Rate limit form state
    pub rl_port_input: String,
    pub rl_protocol: String,
    pub rl_rate_input: String,
    pub rl_unit: String,
    pub rl_focus: usize, // 0: port, 1: rate

    pub message: Option<(bool, String)>, // (is_error, message)
    pub current_table: String,

    // Rollback Timer state
    pub rollback_active: bool,
    pub rollback_deadline: Option<Instant>,
}

impl AppState {
    /// Create a new AppState
    pub fn new() -> Self {
        // En producción el instalador deja los scripts aquí
        let scripts_path = if std::path::Path::new("/usr/local/share/pathogen/scripts").exists() {
            "/usr/local/share/pathogen/scripts"
        } else if std::path::Path::new("/usr/local/share/nftables-tui/scripts").exists() {
            "/usr/local/share/nftables-tui/scripts"
        } else {
            "./scripts" // Fallback local para desarrollo
        };

        let mut app = Self {
            repository: CliFirewallRepository::new(scripts_path),
            mode: AppMode::Menu,
            rules: Vec::new(),
            rules_state: ListState::default(),
            logs: Vec::new(),
            quarantined_ips: Vec::new(),
            selected_index: 0,
            menu_index: 0,
            log_scroll: 0,
            quarantine_index: 0,
            show_block_dialog: false,
            block_port_input: String::new(),
            block_protocol: "tcp".to_string(),
            show_quarantine_dialog: false,
            quarantine_ip_input: String::new(),
            show_rate_limit_dialog: false,
            rl_port_input: String::new(),
            rl_protocol: "tcp".to_string(),
            rl_rate_input: "10".to_string(),
            rl_unit: "minute".to_string(),
            rl_focus: 0,
            message: None,
            current_table: "filter".to_string(),
            rollback_active: false,
            rollback_deadline: None,
            rate_limit_list: RateLimitList::new(),
        };
        app.refresh_rules();
        app
    }

    pub fn start_rollback(&mut self) -> Result<(), String> {
        self.repository.backup_ruleset()?;
        self.rollback_active = true;
        self.rollback_deadline = Some(Instant::now() + Duration::from_secs(30));
        Ok(())
    }

    pub fn confirm_rollback(&mut self) {
        self.rollback_active = false;
        self.rollback_deadline = None;
        self.message = Some((false, "Changes confirmed successfully.".to_string()));
    }

    pub fn cancel_rollback(&mut self) {
        if let Err(e) = self.repository.restore_ruleset() {
            self.message = Some((true, format!("Failed to rollback: {}", e)));
        } else {
            self.message = Some((false, "Reverted safely. No connection lost.".to_string()));
        }
        self.rollback_active = false;
        self.rollback_deadline = None;
        self.refresh_rules();
        self.refresh_quarantine();
        // Clear any pending IPs that were added during the rollback window
        self.quarantined_ips = self.repository.get_quarantined_ips();
    }

    pub fn refresh_rules(&mut self) {
        self.rules = self.repository.get_rules_by_table(&self.current_table);
        if self.selected_index >= self.rules.len() && !self.rules.is_empty() {
            self.selected_index = self.rules.len() - 1;
        }
        // Initialize rules_state for scroll
        if !self.rules.is_empty() {
            self.rules_state.select(Some(0));
        }
    }

    pub fn refresh_logs(&mut self) {
        self.logs = self.repository.get_logs();
        self.log_scroll = 0;
    }

    pub fn refresh_quarantine(&mut self) {
        self.quarantined_ips = self.repository.get_quarantined_ips();
        if self.quarantine_index >= self.quarantined_ips.len() && !self.quarantined_ips.is_empty() {
            self.quarantine_index = self.quarantined_ips.len() - 1;
        }
    }

    pub fn refresh_rate_limit_rules(&mut self) {
        self.rate_limit_list.rate_limits = self.repository.get_rate_limit_rules();
        if let Some(selected) = self.rate_limit_list.state.selected() {
            if selected >= self.rate_limit_list.rate_limits.len()
                && !self.rate_limit_list.rate_limits.is_empty()
            {
                self.rate_limit_list
                    .state
                    .select(Some(self.rate_limit_list.rate_limits.len() - 1));
            }
        }
    }

    pub fn get_rules(&self) -> &Vec<Rule> {
        &self.rules
    }

    pub fn block_port(&mut self) {
        let port: u16 = match self.block_port_input.parse() {
            Ok(p) => p,
            Err(_) => {
                self.message = Some((true, "Puerto inválido".to_string()));
                return;
            }
        };

        if let Err(e) = self.start_rollback() {
            self.message = Some((true, e));
            return;
        }

        let request = PortRequest::new(port, self.block_protocol.clone());

        match self.repository.block_port(request) {
            Ok(_) => {
                self.message = Some((false, format!("Puerto {} bloqueado exitosamente", port)));
                self.show_block_dialog = false;
                self.block_port_input.clear();
                self.refresh_rules();
            }
            Err(e) => {
                self.message = Some((true, e));
                self.cancel_rollback();
            }
        }
    }

    pub fn apply_rate_limit(&mut self) {
        let port: u16 = match self.rl_port_input.parse() {
            Ok(p) => p,
            Err(_) => {
                self.message = Some((true, "Puerto inválido".to_string()));
                return;
            }
        };
        let rate: u32 = match self.rl_rate_input.parse() {
            Ok(r) => r,
            Err(_) => {
                self.message = Some((true, "Tasa inválida".to_string()));
                return;
            }
        };

        if let Err(e) = self.start_rollback() {
            self.message = Some((true, e));
            return;
        }

        let req = RateLimitRequest::new(port, self.rl_protocol.clone(), rate, self.rl_unit.clone());
        match self.repository.apply_rate_limit(req) {
            Ok(_) => {
                self.message = Some((false, format!("Límite aplicado a {}", port)));
                self.rl_port_input.clear();
                self.rl_rate_input = "10".to_string();
                self.show_rate_limit_dialog = false;
                self.refresh_rate_limit_rules();
            }
            Err(e) => {
                self.message = Some((true, e));
                self.cancel_rollback();
            }
        }
    }

    pub fn quarantine_ip(&mut self) {
        use std::str::FromStr;
        if std::net::IpAddr::from_str(&self.quarantine_ip_input).is_err() {
            self.message = Some((true, "Dirección IP inválida".to_string()));
            return;
        }

        if let Err(e) = self.start_rollback() {
            self.message = Some((true, e));
            return;
        }

        let req = QuarantineRequest::new(self.quarantine_ip_input.clone());
        match self.repository.quarantine_ip(req) {
            Ok(_) => {
                self.message = Some((
                    false,
                    format!("IP {} en cuarentena", self.quarantine_ip_input),
                ));
                self.show_quarantine_dialog = false;
                self.quarantine_ip_input.clear();
                self.refresh_quarantine();
            }
            Err(e) => {
                self.message = Some((true, e));
                self.cancel_rollback();
            }
        }
    }

    pub fn remove_quarantine(&mut self) {
        if self.quarantined_ips.is_empty() {
            return;
        }
        if let Some(ip) = self.quarantined_ips.get(self.quarantine_index).cloned() {
            if let Err(e) = self.start_rollback() {
                self.message = Some((true, e));
                return;
            }
            if let Err(e) = self.repository.unquarantine_ip(&ip) {
                self.message = Some((true, e));
                self.cancel_rollback();
            } else {
                self.refresh_quarantine();
                self.message = Some((false, "IP liberada de cuarentena.".to_string()));
            }
        }
    }

    pub fn delete_rule(&mut self) {
        if self.rules.is_empty() {
            return;
        }
        // Use rules_state for deletion
        if let Some(selected) = self.rules_state.selected() {
            if let Some(rule) = self.rules.get(selected).cloned() {
                if rule.action == crate::domain::action::Action::Drop {
                    if let Some(p) = rule.dst_port {
                        if let Err(e) = self.start_rollback() {
                            self.message = Some((true, e));
                            return;
                        }
                        if let Err(e) = self.repository.unblock_port(p) {
                            self.message = Some((true, e));
                            self.cancel_rollback();
                        } else {
                            self.refresh_rules();
                            self.message = Some((false, "Regla eliminada.".to_string()));
                        }
                    }
                } else {
                    self.message = Some((
                        true,
                        "Solo se pueden eliminar bloqueos creados aquí por ahora.".to_string(),
                    ));
                }
            }
        }
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
