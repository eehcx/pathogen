use crate::domain::rule::Rule;
use crate::domain::action::Action;
use crate::domain::port_request::PortRequest;
use crate::domain::rate_limit::RateLimitRequest;
use crate::domain::quarantine::QuarantineRequest;
use crate::use_cases::firewall_trait::FirewallRepository;
use crate::infrastructure::nftables_json::{NftablesOutput, NftablesItem};
use std::process::Command;
use std::path::Path;
use libc;

pub struct CliFirewallRepository {
    scripts_dir: String,
}

impl CliFirewallRepository {
    pub fn new(scripts_dir: &str) -> Self {
        Self {
            scripts_dir: scripts_dir.to_string(),
        }
    }

    fn run_script(&self, script_name: &str, args: &[&str]) -> Result<String, String> {
        let script_path = Path::new(&self.scripts_dir).join(script_name);
        let path_str = script_path.to_str().unwrap();

        // For development, just log what would happen
        println!("[DEV] Would run: {} {}", path_str, args.join(" "));
        
        // Check if we're already root
        let is_root = unsafe { libc::geteuid() == 0 };
        
        let mut cmd = if is_root {
            Command::new(path_str)
        } else {
            let mut cmd = Command::new("sudo");
            cmd.arg("-n").arg(path_str);
            cmd
        };
        
        cmd.args(args);
        
        let output = cmd.output()
            .map_err(|e| format!("Failed to execute script {}: {}", path_str, e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            // For development, simulate success
            if std::env::var("PATHOGEN_DEV_MODE").is_ok() {
                println!("[DEV] Simulating success for: {}", script_name);
                Ok("{\"status\":\"ok\"}".to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
    }
}

impl FirewallRepository for CliFirewallRepository {
    fn get_all_rules(&self) -> Vec<Rule> {
        let json_output = self.run_script("nft_list_rules.sh", &[]).unwrap_or_default();
        let mut parsed_rules = Vec::new();

        if let Ok(parsed) = serde_json::from_str::<NftablesOutput>(&json_output) {
            for item in parsed.nftables {
                if let NftablesItem::RuleWrapper { rule } = item {
                    let mut is_drop = false;
                    let mut protocol = "tcp".to_string();
                    let mut port: Option<u16> = None;

                    if let Some(ref comment) = rule.comment {
                        if comment.starts_with("tui-blocked-") {
                            is_drop = true;
                            let parts: Vec<&str> = comment.split('-').collect();
                            if parts.len() >= 4 {
                                protocol = parts[2].to_string();
                                port = parts[3].parse::<u16>().ok();
                            }
                        } else if comment.starts_with("tui-ratelimit-") {
                            is_drop = true;
                            let parts: Vec<&str> = comment.split('-').collect();
                            if parts.len() >= 4 {
                                protocol = parts[2].to_string();
                                port = parts[3].parse::<u16>().ok();
                            }
                        }
                    }

                    let action = if is_drop { Action::Drop } else { Action::Accept };

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

    fn get_rules_by_table(&self, table: &str) -> Vec<Rule> {
        self.get_all_rules().into_iter().filter(|r| r.table == table).collect()
    }

    fn block_port(&mut self, request: PortRequest) -> Result<Rule, String> {
        if !request.is_valid() {
            return Err(request.validation_error().unwrap_or_else(|| "Invalid request".to_string()));
        }

        let port_str = request.port.to_string();
        self.run_script("nft_block_port.sh", &[&request.protocol, &port_str])?;
        
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

    fn unblock_port(&mut self, port: u16) -> Result<(), String> {
        let port_str = port.to_string();
        let _ = self.run_script("nft_unblock_port.sh", &["tcp", &port_str]);
        let _ = self.run_script("nft_unblock_port.sh", &["udp", &port_str]);
        Ok(())
    }

    fn is_port_blocked(&self, port: u16) -> bool {
        self.get_all_rules().into_iter().any(|r| r.dst_port == Some(port) && r.action == Action::Drop)
    }

    fn get_logs(&self) -> Vec<String> {
        let output = self.run_script("nft_get_logs.sh", &[]).unwrap_or_default();
        output.lines()
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty())
            .collect()
    }

    fn apply_rate_limit(&mut self, request: RateLimitRequest) -> Result<(), String> {
        let port_str = request.port.to_string();
        let rate_str = request.rate.to_string();
        self.run_script("nft_rate_limit.sh", &[&port_str, &request.protocol, &rate_str, &request.unit])?;
        Ok(())
    }

    fn quarantine_ip(&mut self, request: QuarantineRequest) -> Result<(), String> {
        if !request.is_valid() {
            return Err("Dirección IP inválida".to_string());
        }
        self.run_script("nft_quarantine_ip.sh", &[&request.ip])?;
        Ok(())
    }

    fn unquarantine_ip(&mut self, ip: &str) -> Result<(), String> {
        self.run_script("nft_unquarantine_ip.sh", &[ip])?;
        Ok(())
    }

    fn get_quarantined_ips(&self) -> Vec<String> {
        let output = self.run_script("nft_list_quarantine.sh", &[]).unwrap_or_default();
        let mut ips = Vec::new();

        if let Ok(parsed) = serde_json::from_str::<NftablesOutput>(&output) {
            for item in parsed.nftables {
                // Parsing the set elements natively can be tricky, 
                // but we can extract them from the JSON.
                // We will implement a simplified extractor or rely on the JSON shape.
                if let NftablesItem::Unknown(val) = item {
                    if let Some(set) = val.get("set") {
                        if set.get("name").and_then(|n| n.as_str()) == Some("pathogen_quarantine") {
                            if let Some(elem) = set.get("elem").and_then(|e| e.as_array()) {
                                for e in elem {
                                    if let Some(ip) = e.as_str() {
                                        ips.push(ip.to_string());
                                    }
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
        // Direct call to nft to backup
        let mut cmd = Command::new("sudo");
        cmd.arg("-n").arg("sh").arg("-c").arg("nft list ruleset > /tmp/pathogen_backup.nft");
        let output = cmd.output()
            .map_err(|e| format!("Failed to create backup: {}", e))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!("Backup failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    fn restore_ruleset(&self) -> Result<(), String> {
        // Ejecutamos flush ruleset y luego cargamos el archivo de backup en un script atómico
        let mut cmd = Command::new("sudo");
        cmd.arg("-n").arg("sh").arg("-c").arg("nft flush ruleset && nft -f /tmp/pathogen_backup.nft");
        let output = cmd.output()
            .map_err(|e| format!("Failed to restore backup: {}", e))?;
            
        if output.status.success() {
            Ok(())
        } else {
            Err(format!("Restore failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    fn get_rate_limit_rules(&self) -> Vec<String> {
        // Ejecutar nft list ruleset y filtrar reglas de rate limit
        let output = match Command::new("sudo")
            .args(["nft", "list", "ruleset"])
            .output()
        {
            Ok(output) => output,
            Err(_) => return vec!["Error: Could not execute nft command".to_string()],
        };

        if !output.status.success() {
            return vec!["Error: nft command failed".to_string()];
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut rules = Vec::new();
        
        // Buscar reglas que contengan "limit rate" o "meter"
        for line in stdout.lines() {
            if line.contains("limit rate") || line.contains("meter") {
                rules.push(line.trim().to_string());
            }
        }

        if rules.is_empty() {
            rules.push("No rate limit rules found".to_string());
        }

        rules
    }

    fn delete_rate_limit_rule(&self, rule: &str) -> Result<(), String> {
        // Extraer el handle de la regla (último número entre corchetes)
        let handle = rule
            .split_whitespace()
            .find(|part| part.starts_with('#'))
            .map(|h| h.trim_start_matches('#'))
            .unwrap_or("");

        if handle.is_empty() {
            return Err("Could not extract rule handle".to_string());
        }

        // Eliminar la regla usando su handle
        let output = Command::new("sudo")
            .args(["nft", "delete", "rule", "filter", "input", "handle", handle])
            .output()
            .map_err(|e| format!("Failed to delete rule: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to delete rule: {}", stderr))
        }
    }
}
