use crate::domain::action::Action;
use crate::domain::port_request::PortRequest;
use crate::domain::quarantine::QuarantineRequest;
use crate::domain::rate_limit::RateLimitRequest;
use crate::domain::rule::Rule;
use crate::infrastructure::audit_logger::AuditLogger;
use crate::infrastructure::nft_commander::NftCommander;
use crate::infrastructure::nftables_json::{NftablesItem, NftablesOutput};
use crate::use_cases::firewall_trait::FirewallRepository;

pub struct CliFirewallRepository {
    audit_logger: AuditLogger,
    scripts_dir: String,
}

impl CliFirewallRepository {
    pub fn new(scripts_dir: &str) -> Self {
        Self {
            audit_logger: AuditLogger::new(),
            scripts_dir: scripts_dir.to_string(),
        }
    }
}

impl CliFirewallRepository {
    fn run_script(&self, script: &str, args: &[String]) -> Result<String, String> {
        use std::path::Path;
        use std::process::Command;

        let script_path = Path::new(&self.scripts_dir).join(script);
        if !script_path.exists() {
            return Err(format!("Script not found: {}", script_path.display()));
        }

        let output = Command::new(script_path)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute script: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    fn parse_action_from_expr(expr: &[serde_json::Value]) -> Action {
        for e in expr.iter().rev() {
            if let Some(obj) = e.as_object() {
                if obj.contains_key("drop") {
                    return Action::Drop;
                }
                if obj.contains_key("accept") {
                    return Action::Accept;
                }
                if let Some(verdict) = obj.get("verdict").and_then(|v| v.as_str()) {
                    match verdict {
                        "drop" => return Action::Drop,
                        "accept" => return Action::Accept,
                        _ => {}
                    }
                }
            }
        }
        Action::Accept
    }

    fn parse_proto_port_from_expr(expr: &[serde_json::Value]) -> (Option<String>, Option<u16>) {
        for e in expr {
            let match_obj = match e.get("match") {
                Some(m) => m,
                None => continue,
            };
            let left = match match_obj.get("left") {
                Some(l) => l,
                None => continue,
            };
            let payload = match left.get("payload") {
                Some(p) => p,
                None => continue,
            };
            let field = payload.get("field").and_then(|f| f.as_str()).unwrap_or("");
            if field != "dport" {
                continue;
            }
            let protocol = payload
                .get("protocol")
                .and_then(|p| p.as_str())
                .map(|p| p.to_string());
            let right = match_obj.get("right");
            let port = match right {
                Some(v) if v.is_u64() => v.as_u64().and_then(|n| u16::try_from(n).ok()),
                Some(v) if v.is_string() => v.as_str().and_then(|s| s.parse::<u16>().ok()),
                _ => None,
            };
            return (protocol, port);
        }
        (None, None)
    }

    fn parse_rate_limit_from_expr(expr: &[serde_json::Value]) -> Option<(u32, String)> {
        for e in expr {
            let limit = e.get("limit")?;
            let rate = limit.get("rate").and_then(|v| v.as_u64())? as u32;
            let unit = limit
                .get("unit")
                .or_else(|| limit.get("per"))
                .and_then(|v| v.as_str())
                .unwrap_or("second")
                .to_string();
            return Some((rate, unit));
        }
        None
    }

    fn parse_rate_limit_from_comment(comment: &str) -> Option<(String, u16, u32, String)> {
        // Expected format: tui-ratelimit-<proto>-<port>-<rate>-<unit>
        if !comment.starts_with("tui-ratelimit-") {
            return None;
        }
        let parts: Vec<&str> = comment.split('-').collect();
        if parts.len() < 6 {
            return None;
        }
        let protocol = parts[2].to_string();
        let port = parts[3].parse::<u16>().ok()?;
        let rate = parts[4].parse::<u32>().ok()?;
        let unit = parts[5].to_string();
        Some((protocol, port, rate, unit))
    }

    fn list_rules_json(&self) -> Result<String, String> {
        match NftCommander::list_rules() {
            Ok(out) => Ok(out),
            Err(e) => {
                self.audit_logger.log("list_rules", "-", &format!("FALLBACK: {}", e));
                self.run_script("nft_list_rules.sh", &[])
            }
        }
    }
}

impl FirewallRepository for CliFirewallRepository {
    fn get_all_rules(&self) -> Vec<Rule> {
        let json_output = self.list_rules_json().unwrap_or_default();

        let mut parsed_rules = Vec::new();

        if json_output.is_empty() {
            return parsed_rules;
        }

        if let Ok(parsed) = serde_json::from_str::<NftablesOutput>(&json_output) {
            for item in parsed.nftables {
                if let NftablesItem::Rule { rule } = item {
                    let action = Self::parse_action_from_expr(&rule.expr);

                    let (expr_proto, expr_port) = Self::parse_proto_port_from_expr(&rule.expr);
                    let mut protocol = expr_proto.unwrap_or_else(|| "tcp".to_string());
                    let mut port: Option<u16> = expr_port;

                    // Extract port from tui-blocked-* comments
                    if let Some(ref comment) = rule.comment {
                        if comment.starts_with("tui-blocked-") {
                            let parts: Vec<&str> = comment.split('-').collect();
                            if parts.len() >= 4 {
                                protocol = parts[2].to_string();
                                port = parts[3].parse::<u16>().ok();
                            }
                        }
                    }

                    let domain_rule = Rule {
                        table: rule.table.clone(),
                        chain: rule.chain.clone(),
                        priority: rule.handle as i32,
                        action,
                        protocol,
                        src_port: None,
                        dst_port: port,
                        enabled: true,
                        comment: rule.comment.clone(),
                    };
                    parsed_rules.push(domain_rule);
                }
            }
        }

        parsed_rules
    }

    fn get_rules_by_table(&self, _table: &str) -> Vec<Rule> {
        // Return all rules - don't filter by table for maximum compatibility
        // with different nftables setups (firewalld, custom tables, etc.)
        self.get_all_rules()
    }

    fn block_port(&mut self, request: PortRequest) -> Result<Rule, String> {
        if !request.is_valid() {
            self.audit_logger.log(
                "block_port",
                &format!("port={}", request.port),
                "REJECTED: invalid request",
            );
            return Err(request
                .validation_error()
                .unwrap_or_else(|| "Invalid request".to_string()));
        }

        let comment = format!("tui-blocked-{}-{}", request.protocol, request.port);
        match NftCommander::block_port(&request.protocol, request.port, &comment) {
            Ok(_) => {
                self.audit_logger.log(
                    "block_port",
                    &format!("protocol={} port={}", request.protocol, request.port),
                    "SUCCESS",
                );
                Ok(Rule {
                    table: "filter".to_string(),
                    chain: "input".to_string(),
                    priority: 0,
                    action: Action::Drop,
                    protocol: request.protocol.clone(),
                    src_port: None,
                    dst_port: Some(request.port),
                    enabled: true,
                    comment: Some(format!("tui-blocked-{}-{}", request.protocol, request.port)),
                })
            }
            Err(_e) => {
                let args = vec![request.protocol.clone(), request.port.to_string()];
                match self.run_script("nft_block_port.sh", &args) {
                    Ok(_) => {
                        self.audit_logger.log(
                            "block_port",
                            &format!("protocol={} port={}", request.protocol, request.port),
                            "FALLBACK_SUCCESS",
                        );
                        return Ok(Rule {
                            table: "filter".to_string(),
                            chain: "input".to_string(),
                            priority: 0,
                            action: Action::Drop,
                            protocol: request.protocol.clone(),
                            src_port: None,
                            dst_port: Some(request.port),
                            enabled: true,
                            comment: Some(format!(
                                "tui-blocked-{}-{}",
                                request.protocol, request.port
                            )),
                        });
                    }
                    Err(fallback_err) => {
                        self.audit_logger.log(
                            "block_port",
                            &format!("protocol={} port={}", request.protocol, request.port),
                            &format!("ERROR: {}", fallback_err),
                        );
                        return Err(fallback_err);
                    }
                }
            }
        }
    }

    fn unblock_port(&mut self, port: u16) -> Result<(), String> {
        // Find and delete the blocking rule by looking for tui-blocked-* comment
        let rules = self.get_all_rules();
        let mut last_error: Option<String> = None;

        for rule in rules
            .iter()
            .filter(|r| r.dst_port == Some(port) && r.action == Action::Drop)
        {
            // Get the handle from priority (it's stored there)
            let handle = rule.priority as u64;
            if handle > 0 {
                match NftCommander::delete_rule("filter", "input", handle) {
                    Ok(_) => {
                        self.audit_logger
                            .log("unblock_port", &format!("port={}", port), "SUCCESS");
                        return Ok(());
                    }
                    Err(e) => {
                        let args = vec![rule.protocol.clone(), port.to_string()];
                        match self.run_script("nft_unblock_port.sh", &args) {
                            Ok(_) => {
                                self.audit_logger.log(
                                    "unblock_port",
                                    &format!("port={}", port),
                                    "FALLBACK_SUCCESS",
                                );
                                return Ok(());
                            }
                            Err(fallback_err) => {
                                last_error = Some(format!("{}; {}", e, fallback_err));
                            }
                        }
                    }
                }
            } else {
                let args = vec![rule.protocol.clone(), port.to_string()];
                match self.run_script("nft_unblock_port.sh", &args) {
                    Ok(_) => {
                        self.audit_logger
                            .log("unblock_port", &format!("port={}", port), "FALLBACK_SUCCESS");
                        return Ok(());
                    }
                    Err(e) => last_error = Some(e),
                }
            }
        }

        let err_msg = last_error.unwrap_or_else(|| format!("Port {} not found in rules", port));
        self.audit_logger.log(
            "unblock_port",
            &format!("port={}", port),
            &format!("ERROR: {}", err_msg),
        );
        Err(err_msg)
    }

    fn is_port_blocked(&self, port: u16) -> bool {
        self.get_all_rules()
            .into_iter()
            .any(|r| r.dst_port == Some(port) && r.action == Action::Drop)
    }

    fn get_logs(&self) -> Vec<String> {
        // For now, return a simple message - full log implementation would need
        // kernel audit logs or nft counter interpretation
        vec!["Log viewing requires additional setup (auditd)".to_string()]
    }

    fn apply_rate_limit(&mut self, request: RateLimitRequest) -> Result<(), String> {
        match NftCommander::add_rate_limit(
            &request.protocol,
            request.port,
            request.rate,
            &request.unit,
        ) {
            Ok(_) => {
                self.audit_logger.log(
                    "apply_rate_limit",
                    &format!(
                        "port={} rate={}/{}",
                        request.port, request.rate, request.unit
                    ),
                    "SUCCESS",
                );
                Ok(())
            }
            Err(e) => {
                let args = vec![
                    request.port.to_string(),
                    request.protocol.clone(),
                    request.rate.to_string(),
                    request.unit.clone(),
                ];
                if self.run_script("nft_rate_limit.sh", &args).is_ok() {
                    self.audit_logger.log(
                        "apply_rate_limit",
                        &format!(
                            "port={} rate={}/{}",
                            request.port, request.rate, request.unit
                        ),
                        "FALLBACK_SUCCESS",
                    );
                    return Ok(());
                }
                self.audit_logger.log(
                    "apply_rate_limit",
                    &format!(
                        "port={} rate={}/{}",
                        request.port, request.rate, request.unit
                    ),
                    &format!("ERROR: {}", e),
                );
                Err(e)
            }
        }
    }

    fn quarantine_ip(&mut self, request: QuarantineRequest) -> Result<(), String> {
        if !request.is_valid() {
            self.audit_logger.log(
                "quarantine_ip",
                &format!("ip={}", request.ip),
                "REJECTED: invalid IP",
            );
            return Err("Dirección IP inválida".to_string());
        }
        match NftCommander::add_to_quarantine(&request.ip) {
            Ok(_) => {
                self.audit_logger
                    .log("quarantine_ip", &format!("ip={}", request.ip), "SUCCESS");
                Ok(())
            }
            Err(e) => {
                let args = vec![request.ip.clone()];
                if self.run_script("nft_quarantine_ip.sh", &args).is_ok() {
                    self.audit_logger
                        .log("quarantine_ip", &format!("ip={}", request.ip), "FALLBACK_SUCCESS");
                    return Ok(());
                }
                self.audit_logger.log(
                    "quarantine_ip",
                    &format!("ip={}", request.ip),
                    &format!("ERROR: {}", e),
                );
                Err(e)
            }
        }
    }

    fn unquarantine_ip(&mut self, ip: &str) -> Result<(), String> {
        match NftCommander::remove_from_quarantine(ip) {
            Ok(_) => {
                self.audit_logger
                    .log("unquarantine_ip", &format!("ip={}", ip), "SUCCESS");
                Ok(())
            }
            Err(e) => {
                let args = vec![ip.to_string()];
                if self.run_script("nft_unquarantine_ip.sh", &args).is_ok() {
                    self.audit_logger
                        .log("unquarantine_ip", &format!("ip={}", ip), "FALLBACK_SUCCESS");
                    return Ok(());
                }
                self.audit_logger.log(
                    "unquarantine_ip",
                    &format!("ip={}", ip),
                    &format!("ERROR: {}", e),
                );
                Err(e)
            }
        }
    }

    fn get_quarantined_ips(&self) -> Vec<String> {
        let output = match NftCommander::list_quarantine() {
            Ok(o) => o,
            Err(e) => {
                self.audit_logger
                    .log("list_quarantine", "-", &format!("FALLBACK: {}", e));
                self.run_script("nft_list_quarantine.sh", &[])
                    .unwrap_or_default()
            }
        };
        let mut ips = Vec::new();

        if let Ok(parsed) = serde_json::from_str::<NftablesOutput>(&output) {
            for item in parsed.nftables {
                if let NftablesItem::Set { set } = item {
                    if set.name == "pathogen_quarantine" {
                        for elem in set.elem {
                            if let Some(ip) = elem.as_str() {
                                ips.push(ip.to_string());
                            } else if let Some(obj) = elem.as_object() {
                                if let Some(ip) = obj.get("elem").and_then(|v| v.as_str()) {
                                    ips.push(ip.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        ips
    }

    fn backup_ruleset(&self) -> Result<(), String> {
        let output =
            NftCommander::list_rules().map_err(|e| format!("Failed to create backup: {}", e))?;

        std::fs::write("/tmp/pathogen_backup.nft", &output)
            .map_err(|e| format!("Failed to write backup file: {}", e))?;

        self.audit_logger
            .log("backup_ruleset", "/tmp/pathogen_backup.nft", "SUCCESS");
        Ok(())
    }

    fn restore_ruleset(&self) -> Result<(), String> {
        // First flush current rules
        NftCommander::flush_ruleset().map_err(|e| format!("Failed to flush ruleset: {}", e))?;

        // Then restore from backup
        NftCommander::restore_ruleset("/tmp/pathogen_backup.nft")
            .map_err(|e| format!("Failed to restore backup: {}", e))?;

        self.audit_logger
            .log("restore_ruleset", "/tmp/pathogen_backup.nft", "SUCCESS");
        Ok(())
    }

    fn get_rate_limit_rules(&self) -> Vec<String> {
        let output = match self.list_rules_json() {
            Ok(output) => output,
            Err(_) => return vec!["Error: Could not execute nft command".to_string()],
        };

        let mut rules = Vec::new();
        if let Ok(parsed) = serde_json::from_str::<NftablesOutput>(&output) {
            for item in parsed.nftables {
                if let NftablesItem::Rule { rule } = item {
                    let limit = Self::parse_rate_limit_from_expr(&rule.expr);
                    let is_rl = rule
                        .comment
                        .as_deref()
                        .map(|c| c.starts_with("tui-ratelimit-"))
                        .unwrap_or(false)
                        || limit.is_some();
                    if !is_rl {
                        continue;
                    }
                    let (mut protocol, mut port) = Self::parse_proto_port_from_expr(&rule.expr);
                    let mut rate = limit.as_ref().map(|v| v.0).unwrap_or(0);
                    let mut unit = limit
                        .as_ref()
                        .map(|v| v.1.clone())
                        .unwrap_or_else(|| "second".to_string());

                    if let Some(comment) = rule.comment.as_deref() {
                        if let Some((c_proto, c_port, c_rate, c_unit)) =
                            Self::parse_rate_limit_from_comment(comment)
                        {
                            if protocol.is_none() {
                                protocol = Some(c_proto);
                            }
                            if port.is_none() {
                                port = Some(c_port);
                            }
                            if rate == 0 {
                                rate = c_rate;
                            }
                            if unit == "second" {
                                unit = c_unit;
                            }
                        }
                    }

                    let protocol = protocol.unwrap_or_else(|| "tcp".to_string());
                    let port_str = port.map(|p| p.to_string()).unwrap_or_else(|| "*".to_string());
                    rules.push(format!(
                        "{} dport {} limit rate {}/{} #{}",
                        protocol, port_str, rate, unit, rule.handle
                    ));
                }
            }
        }

        if rules.is_empty() {
            rules.push("No rate limit rules found".to_string());
        }
        rules
    }

    fn delete_rate_limit_rule(&self, rule: &str) -> Result<(), String> {
        let handle = rule
            .split_whitespace()
            .find_map(|part| {
                if part.starts_with('#') {
                    Some(part.trim_start_matches('#'))
                } else {
                    None
                }
            })
            .or_else(|| {
                let mut iter = rule.split_whitespace();
                while let Some(part) = iter.next() {
                    if part == "handle" {
                        return iter.next();
                    }
                }
                None
            })
            .unwrap_or("");

        if handle.is_empty() {
            return Err("Could not extract rule handle".to_string());
        }

        match NftCommander::delete_rule("filter", "input", handle.parse().unwrap_or(0)) {
            Ok(_) => {
                self.audit_logger.log(
                    "delete_rate_limit_rule",
                    &format!("handle={}", handle),
                    "SUCCESS",
                );
                Ok(())
            }
            Err(e) => {
                self.audit_logger.log(
                    "delete_rate_limit_rule",
                    &format!("handle={}", handle),
                    &format!("ERROR: {}", e),
                );
                Err(e)
            }
        }
    }
}
