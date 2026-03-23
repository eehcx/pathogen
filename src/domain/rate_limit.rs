pub struct RateLimitRequest {
    pub port: u16,
    pub protocol: String,
    pub rate: u32,
    pub unit: String,
}

impl RateLimitRequest {
    pub fn new(port: u16, protocol: String, rate: u32, unit: String) -> Self {
        Self { port, protocol, rate, unit }
    }
}
