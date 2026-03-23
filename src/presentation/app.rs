use crate::domain::{Rule, PortRequest, QuarantineRequest, RateLimitRequest};
use crate::infrastructure::CliFirewallRepository;
use crate::use_cases::FirewallRepository;

#[derive(PartialEq)]
pub enum AppMode {
    Menu,
    RulesList,
    LogsViewer,
    QuarantineList,
    RateLimitForm,
}

/// AppState represents the application state
pub struct AppState {
    pub repository: CliFirewallRepository,
    pub mode: AppMode,
    pub rules: Vec<Rule>,
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
    
    // Rate limit form state
    pub rl_port_input: String,
    pub rl_protocol: String,
    pub rl_rate_input: String,
    pub rl_unit: String,
    pub rl_focus: usize, // 0: port, 1: rate
    
    pub message: Option<(bool, String)>, // (is_error, message)
    pub current_table: String,
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
            rl_port_input: String::new(),
            rl_protocol: "tcp".to_string(),
            rl_rate_input: "10".to_string(),
            rl_unit: "minute".to_string(),
            rl_focus: 0,
            message: None,
            current_table: "filter".to_string(),
        };
        app.refresh_rules();
        app
    }

    pub fn refresh_rules(&mut self) {
        self.rules = self.repository.get_rules_by_table(&self.current_table);
        if self.selected_index >= self.rules.len() && !self.rules.is_empty() {
            self.selected_index = self.rules.len() - 1;
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
            }
        }
    }

    pub fn apply_rate_limit(&mut self) {
        let port: u16 = match self.rl_port_input.parse() {
            Ok(p) => p,
            Err(_) => { self.message = Some((true, "Puerto inválido".to_string())); return; }
        };
        let rate: u32 = match self.rl_rate_input.parse() {
            Ok(r) => r,
            Err(_) => { self.message = Some((true, "Tasa inválida".to_string())); return; }
        };

        let req = RateLimitRequest::new(port, self.rl_protocol.clone(), rate, self.rl_unit.clone());
        match self.repository.apply_rate_limit(req) {
            Ok(_) => {
                self.message = Some((false, format!("Límite aplicado a {}", port)));
                self.rl_port_input.clear();
                self.rl_rate_input = "10".to_string();
                self.mode = AppMode::Menu;
            }
            Err(e) => self.message = Some((true, e)),
        }
    }

    pub fn quarantine_ip(&mut self) {
        let req = QuarantineRequest::new(self.quarantine_ip_input.clone());
        match self.repository.quarantine_ip(req) {
            Ok(_) => {
                self.message = Some((false, format!("IP {} en cuarentena", self.quarantine_ip_input)));
                self.show_quarantine_dialog = false;
                self.quarantine_ip_input.clear();
                self.refresh_quarantine();
            }
            Err(e) => self.message = Some((true, e)),
        }
    }

    pub fn remove_quarantine(&mut self) {
        if self.quarantined_ips.is_empty() { return; }
        if let Some(ip) = self.quarantined_ips.get(self.quarantine_index) {
            let _ = self.repository.unquarantine_ip(ip);
            self.refresh_quarantine();
            self.message = Some((false, "IP liberada de cuarentena.".to_string()));
        }
    }

    pub fn delete_rule(&mut self) {
        if self.rules.is_empty() { return; }
        if let Some(rule) = self.rules.get(self.selected_index) {
            if rule.action == crate::domain::action::Action::Drop {
                if let Some(p) = rule.dst_port {
                    let _ = self.repository.unblock_port(p);
                    self.refresh_rules();
                    self.message = Some((false, "Regla eliminada.".to_string()));
                }
            } else {
                self.message = Some((true, "Solo se pueden eliminar bloqueos creados aquí por ahora.".to_string()));
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
