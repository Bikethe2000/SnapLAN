#[derive(Debug, Clone)]
pub struct Config {
    pub device_name: String,
    pub port:  u16,
    pub discovery_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device_name: "SnapLAN Device".to_string(),
            port: 9000,
            discovery_port: 8888,
            
        }
    }
}