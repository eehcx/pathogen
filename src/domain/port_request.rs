#![allow(dead_code)]
/// PortRequest represents a request to block or unblock a port
#[derive(Debug, Clone)]
pub struct PortRequest {
    pub port: u16,
    pub protocol: String,
}

impl PortRequest {
    /// Create a new port request
    pub fn new(port: u16, protocol: String) -> Self {
        Self { port, protocol }
    }

    /// Validate the port number (1-65535)
    pub fn is_valid(&self) -> bool {
        self.port >= 1
    }

    /// Get validation error message if invalid
    pub fn validation_error(&self) -> Option<String> {
        if self.port == 0 {
            Some("Puerto inválido. Ingrese un valor entre 1 y 65535".to_string())
        } else if self.protocol != "tcp" && self.protocol != "udp" {
            Some("Protocolo inválido. Use tcp o udp".to_string())
        } else {
            None
        }
    }
}

impl std::fmt::Display for PortRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.protocol, self.port)
    }
}
