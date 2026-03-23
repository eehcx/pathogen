#![allow(dead_code)]
use crate::domain::{Action, PortRequest, Rule};
use crate::use_cases::FirewallRepository;

/// MockFirewallRepository provides fake firewall data for testing
pub struct MockFirewallRepository {
    rules: Vec<Rule>,
}

impl MockFirewallRepository {
    /// Create a new MockFirewallRepository with default rules
    pub fn new() -> Self {
        let mut rules = Vec::new();

        // Add some example rules
        rules.push(Rule::new(
            "filter".to_string(),
            "input".to_string(),
            0,
            Action::Accept,
            "tcp".to_string(),
            None,
            Some(22),
            true,
        ));

        rules.push(Rule::new(
            "filter".to_string(),
            "input".to_string(),
            1,
            Action::Accept,
            "tcp".to_string(),
            None,
            Some(80),
            true,
        ));

        rules.push(Rule::new(
            "filter".to_string(),
            "input".to_string(),
            2,
            Action::Accept,
            "tcp".to_string(),
            None,
            Some(443),
            true,
        ));

        rules.push(Rule::new(
            "filter".to_string(),
            "input".to_string(),
            3,
            Action::Accept,
            "udp".to_string(),
            None,
            Some(53),
            true,
        ));

        rules.push(Rule::new(
            "filter".to_string(),
            "forward".to_string(),
            0,
            Action::Drop,
            "tcp".to_string(),
            None,
            Some(8080),
            true,
        ));

        rules.push(Rule::with_comment(
            "filter".to_string(),
            "input".to_string(),
            4,
            Action::Accept,
            "tcp".to_string(),
            None,
            Some(3306),
            true,
            "MySQL".to_string(),
        ));

        Self { rules }
    }

    /// Create a new empty MockFirewallRepository
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }
}

impl Default for MockFirewallRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl FirewallRepository for MockFirewallRepository {
    fn get_all_rules(&self) -> Vec<Rule> {
        self.rules.clone()
    }

    fn get_rules_by_table(&self, table: &str) -> Vec<Rule> {
        self.rules
            .iter()
            .filter(|r| r.table == table)
            .cloned()
            .collect()
    }

    fn block_port(&mut self, request: PortRequest) -> Result<Rule, String> {
        // Check if port is already blocked
        if self.is_port_blocked(request.port) {
            return Err("El puerto ya está bloqueado".to_string());
        }

        // Validate port
        if let Some(error) = request.validation_error() {
            return Err(error);
        }

        // Create new DROP rule
        let rule = Rule::with_comment(
            "filter".to_string(),
            "input".to_string(),
            self.rules.len() as i32,
            Action::Drop,
            request.protocol.clone(),
            None,
            Some(request.port),
            true,
            format!("Blocked via TUI - {}", request),
        );

        self.rules.push(rule.clone());
        Ok(rule)
    }

    fn unblock_port(&mut self, port: u16) -> Result<(), String> {
        let initial_len = self.rules.len();

        self.rules.retain(|r| {
            // Keep rules that are NOT DROP rules for this port
            !(r.action == Action::Drop && r.dst_port == Some(port))
        });

        if self.rules.len() == initial_len {
            Err("No se encontró una regla de bloqueo para este puerto".to_string())
        } else {
            Ok(())
        }
    }

    fn is_port_blocked(&self, port: u16) -> bool {
        self.rules
            .iter()
            .any(|r| r.action == Action::Drop && r.dst_port == Some(port))
    }

    fn get_logs(&self) -> Vec<String> {
        vec![
            "May 12 10:00:00 pathogen-drop: IN=eth0 OUT= MAC=... SRC=192.168.1.100 DST=10.0.0.5 LEN=60 TOS=0x00 PREC=0x00 TTL=64 ID=50442 DF PROTO=TCP SPT=49228 DPT=22".to_string(),
            "May 12 10:05:22 pathogen-drop: IN=eth0 OUT= MAC=... SRC=203.0.113.42 DST=10.0.0.5 LEN=40 TOS=0x00 PREC=0x00 TTL=245 ID=54321 PROTO=TCP SPT=33412 DPT=80".to_string(),
        ]
    }

    fn apply_rate_limit(&mut self, _request: crate::domain::RateLimitRequest) -> Result<(), String> {
        Ok(())
    }

    fn quarantine_ip(&mut self, request: crate::domain::QuarantineRequest) -> Result<(), String> {
        if !request.is_valid() {
            return Err("IP inválida".to_string());
        }
        Ok(())
    }

    fn unquarantine_ip(&mut self, _ip: &str) -> Result<(), String> {
        Ok(())
    }

    fn get_quarantined_ips(&self) -> Vec<String> {
        vec!["192.168.1.100".to_string(), "203.0.113.42".to_string()]
    }
}
