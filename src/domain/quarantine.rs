pub struct QuarantineRequest {
    pub ip: String,
}

impl QuarantineRequest {
    pub fn new(ip: String) -> Self {
        Self { ip }
    }
    pub fn is_valid(&self) -> bool {
        // Basic IPv4 validation
        let parts: Vec<&str> = self.ip.split('.').collect();
        if parts.len() != 4 { return false; }
        for p in parts {
            if p.parse::<u8>().is_err() { return false; }
        }
        true
    }
}
