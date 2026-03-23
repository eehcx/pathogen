use super::action::Action;

/// Rule represents a firewall rule
#[derive(Debug, Clone)]
pub struct Rule {
    pub table: String,
    pub chain: String,
    pub priority: i32,
    pub action: Action,
    pub protocol: String,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub enabled: bool,
    pub comment: Option<String>,
}

impl Rule {
    /// Create a new rule
    pub fn new(
        table: String,
        chain: String,
        priority: i32,
        action: Action,
        protocol: String,
        src_port: Option<u16>,
        dst_port: Option<u16>,
        enabled: bool,
    ) -> Self {
        Self {
            table,
            chain,
            priority,
            action,
            protocol,
            src_port,
            dst_port,
            enabled,
            comment: None,
        }
    }

    /// Create a new rule with a comment
    pub fn with_comment(
        table: String,
        chain: String,
        priority: i32,
        action: Action,
        protocol: String,
        src_port: Option<u16>,
        dst_port: Option<u16>,
        enabled: bool,
        comment: String,
    ) -> Self {
        Self {
            table,
            chain,
            priority,
            action,
            protocol,
            src_port,
            dst_port,
            enabled,
            comment: Some(comment),
        }
    }
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src = self
            .src_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "*".to_string());
        let dst = self
            .dst_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "*".to_string());

        write!(
            f,
            "{} | {} | {} | {} | {}:{} -> {}",
            self.table,
            self.chain,
            self.action,
            self.protocol,
            src,
            dst,
            if self.enabled { "active" } else { "inactive" }
        )
    }
}
