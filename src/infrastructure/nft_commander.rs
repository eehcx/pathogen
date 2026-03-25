//! NftCommander - Direct nftables interaction using the nftables Rust crate
//! Uses JSON API via the nftables crate

use nftables::helper;
use serde_json::json;
use std::process::Command;

/// NftCommander executes nft commands using the nftables Rust crate
pub struct NftCommander;

impl NftCommander {
    /// Get current ruleset as JSON string
    pub fn list_rules() -> Result<String, String> {
        let ruleset =
            helper::get_current_ruleset().map_err(|e| format!("Failed to get ruleset: {}", e))?;

        serde_json::to_string_pretty(&ruleset).map_err(|e| format!("Failed to serialize: {}", e))
    }

    /// Apply a ruleset from JSON Value
    fn apply_json(value: &serde_json::Value) -> Result<(), String> {
        let json_str =
            serde_json::to_string(value).map_err(|e| format!("Failed to serialize: {}", e))?;

        // Use apply_ruleset instead which takes Nftables struct
        let ruleset: nftables::schema::Nftables =
            serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        nftables::helper::apply_ruleset(&ruleset)
            .map_err(|e| format!("Failed to apply ruleset: {}", e))?;

        Ok(())
    }

    fn apply_statement(statement: serde_json::Value) -> Result<(), String> {
        let ruleset = json!({ "nftables": [statement] });
        Self::apply_json(&ruleset)
    }

    fn is_already_exists_error(err: &str) -> bool {
        let err_lower = err.to_lowercase();
        err_lower.contains("file exists")
            || err_lower.contains("already exists")
            || err_lower.contains("exists")
            || err_lower.contains("eexist")
    }

    fn try_apply(statement: serde_json::Value) -> Result<(), String> {
        match Self::apply_statement(statement) {
            Ok(()) => Ok(()),
            Err(e) if Self::is_already_exists_error(&e) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Add a block rule for a port
    pub fn block_port(protocol: &str, port: u16, comment: &str) -> Result<String, String> {
        Self::try_apply(json!({ "add": { "table": { "family": "inet", "name": "filter" } } }))?;
        Self::try_apply(json!({ "add": { "chain": {
            "family": "inet",
            "table": "filter",
            "name": "input",
            "type": "filter",
            "hook": "input",
            "prio": 0,
            "policy": "accept"
        }}}))?;

        Self::apply_json(&json!({
            "nftables": [
                { "add": { "rule": {
                    "family": "inet",
                    "table": "filter",
                    "chain": "input",
                    "expr": [
                        { "match": {
                            "left": { "payload": { "protocol": protocol, "field": "dport" } },
                            "op": "==",
                            "right": port
                        }},
                        { "log": { "prefix": "pathogen-drop: " } },
                        { "counter": null },
                        { "drop": null }
                    ],
                    "comment": comment
                }}}
            ]
        }))?;
        Ok("Rule added".to_string())
    }

    /// Delete a rule by handle
    pub fn delete_rule(table: &str, chain: &str, handle: u64) -> Result<String, String> {
        let ruleset = json!({
            "nftables": [
                { "delete": { "rule": {
                    "family": "inet",
                    "table": table,
                    "chain": chain,
                    "handle": handle
                }}}
            ]
        });

        Self::apply_json(&ruleset)?;
        Ok("Rule deleted".to_string())
    }

    /// Add a rate limit rule
    pub fn add_rate_limit(
        protocol: &str,
        port: u16,
        rate: u32,
        unit: &str,
    ) -> Result<String, String> {
        let comment = format!("tui-ratelimit-{}-{}-{}-{}", protocol, port, rate, unit);

        Self::try_apply(json!({ "add": { "table": { "family": "inet", "name": "filter" } } }))?;
        Self::try_apply(json!({ "add": { "chain": {
            "family": "inet",
            "table": "filter",
            "name": "input",
            "type": "filter",
            "hook": "input",
            "prio": 0,
            "policy": "accept"
        }}}))?;

        Self::apply_json(&json!({
            "nftables": [
                { "add": { "rule": {
                    "family": "inet",
                    "table": "filter",
                    "chain": "input",
                    "expr": [
                        { "match": {
                            "left": { "payload": { "protocol": protocol, "field": "dport" } },
                            "op": "==",
                            "right": port
                        }},
                        { "match": {
                            "left": { "ct": { "key": "state" } },
                            "op": "==",
                            "right": ["new"]
                        }},
                        { "limit": { "rate": rate, "unit": unit } },
                        { "log": { "prefix": "pathogen-ratelimit: " } },
                        { "counter": null },
                        { "drop": null }
                    ],
                    "comment": comment
                }}}
            ]
        }))?;
        Ok("Rate limit added".to_string())
    }

    /// Add an IP to quarantine
    pub fn add_to_quarantine(ip: &str) -> Result<String, String> {
        Self::try_apply(json!({ "add": { "table": { "family": "inet", "name": "filter" } } }))?;
        Self::try_apply(json!({ "add": { "chain": {
            "family": "inet",
            "table": "filter",
            "name": "input",
            "type": "filter",
            "hook": "input",
            "prio": 0,
            "policy": "accept"
        }}}))?;
        Self::try_apply(json!({ "add": { "set": {
            "family": "inet",
            "table": "filter",
            "name": "pathogen_quarantine",
            "type": "ipv4_addr",
            "flags": ["interval"]
        }}}))?;
        let quarantine_rule = json!({
            "add": { "rule": {
                "family": "inet",
                "table": "filter",
                "chain": "input",
                "expr": [
                    { "lookup": {
                        "set": "pathogen_quarantine",
                        "source": { "payload": { "protocol": "ip", "field": "saddr" } }
                    }},
                    { "log": { "prefix": "pathogen-quarantine: " } },
                    { "counter": null },
                    { "drop": null }
                ],
                "comment": "pathogen-quarantine-rule"
            }}
        });
        let _ = Self::try_apply(quarantine_rule);

        Self::try_apply(json!({ "add": { "element": {
            "family": "inet",
            "table": "filter",
            "set": "pathogen_quarantine",
            "elem": [ip]
        }}}))?;
        Ok("IP added to quarantine".to_string())
    }

    /// Remove an IP from quarantine
    pub fn remove_from_quarantine(ip: &str) -> Result<String, String> {
        let ruleset = json!({
            "nftables": [
                { "delete": { "element": {
                    "family": "inet",
                    "table": "filter",
                    "set": "pathogen_quarantine",
                    "elem": [ip]
                }}}
            ]
        });

        Self::apply_json(&ruleset)?;
        Ok("IP removed from quarantine".to_string())
    }

    /// List quarantine
    pub fn list_quarantine() -> Result<String, String> {
        let output = Command::new("nft")
            .args(["-j", "list", "set", "inet", "filter", "pathogen_quarantine"])
            .output()
            .map_err(|e| format!("Failed to execute nft: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Flush ruleset
    pub fn flush_ruleset() -> Result<String, String> {
        let ruleset = json!({
            "nftables": [
                { "flush": { "table": { "family": "inet", "name": "filter" } } }
            ]
        });

        Self::apply_json(&ruleset)?;
        Ok("Ruleset flushed".to_string())
    }

    /// Restore from file
    pub fn restore_ruleset(path: &str) -> Result<String, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Parse as Nftables
        let ruleset: nftables::schema::Nftables =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse: {}", e))?;

        helper::apply_ruleset(&ruleset).map_err(|e| format!("Failed to restore: {}", e))?;

        Ok("Ruleset restored".to_string())
    }
}
