#![allow(dead_code)]
use crate::domain::{PortRequest, Rule, RateLimitRequest, QuarantineRequest};

/// FirewallRepository defines the interface for firewall operations
pub trait FirewallRepository: Send + Sync {
    /// Get all firewall rules
    fn get_all_rules(&self) -> Vec<Rule>;

    /// Get rules for a specific table
    fn get_rules_by_table(&self, table: &str) -> Vec<Rule>;

    /// Block a port (create a DROP rule)
    fn block_port(&mut self, request: PortRequest) -> Result<Rule, String>;

    /// Unblock a port (remove DROP rules for that port)
    fn unblock_port(&mut self, port: u16) -> Result<(), String>;

    /// Check if a port is already blocked
    fn is_port_blocked(&self, port: u16) -> bool;

    /// Get firewall logs
    fn get_logs(&self) -> Vec<String>;

    /// Apply a rate limit to a port
    fn apply_rate_limit(&mut self, request: RateLimitRequest) -> Result<(), String>;

    /// Quarantine an IP address
    fn quarantine_ip(&mut self, request: QuarantineRequest) -> Result<(), String>;

    /// Remove an IP address from quarantine
    fn unquarantine_ip(&mut self, ip: &str) -> Result<(), String>;

    /// Get all quarantined IPs
    fn get_quarantined_ips(&self) -> Vec<String>;
}
