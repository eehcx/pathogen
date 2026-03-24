use chrono::{DateTime, Local};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// Audit logger for tracking firewall operations
pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    /// Create a new audit logger with fallback paths
    pub fn new() -> Self {
        let log_path = Self::find_log_path();
        Self { log_path }
    }

    /// Find the best available log path with fallbacks
    fn find_log_path() -> PathBuf {
        // Try /var/log/pathogen first (if writable)
        let system_log = PathBuf::from("/var/log/pathogen/audit.log");

        // Try to create the directory and log file
        if let Some(parent) = system_log.parent() {
            if fs::create_dir_all(parent).is_ok() {
                if OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&system_log)
                    .is_ok()
                {
                    return system_log;
                }
            }
        }

        // Fallback to user home directory
        if let Some(dirs) = directories::ProjectDirs::from("com", "pathogen", "pathogen") {
            let user_log = dirs.data_dir().join("audit.log");
            if let Some(parent) = user_log.parent() {
                let _ = fs::create_dir_all(parent);
                if OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&user_log)
                    .is_ok()
                {
                    return user_log;
                }
            }
        }

        // Last resort: current directory
        PathBuf::from("pathogen_audit.log")
    }

    /// Log an audit event
    pub fn log(&self, action: &str, details: &str, result: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let username = Self::get_username();

        let log_entry = format!(
            "[{}] USER={} ACTION={} DETAILS=\"{}\" RESULT={}\n",
            timestamp, username, action, details, result
        );

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }
    }

    /// Get current username
    fn get_username() -> String {
        // Try USER env var first
        if let Ok(user) = std::env::var("USER") {
            if !user.is_empty() {
                return user;
            }
        }

        // Fallback to whoami command
        if let Ok(output) = Command::new("whoami").output() {
            if let Ok(username) = String::from_utf8(output.stdout) {
                return username.trim().to_string();
            }
        }

        "unknown".to_string()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
