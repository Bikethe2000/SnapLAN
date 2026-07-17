pub const APP_NAME: &str = "SnapLAN";
pub const PROTOCOL_VERSION: u16 = 1;

pub const DISCOVERY_PORT: u16 = 45454;
pub const DISCOVERY_INTERVAL_WS: u64 = 1000;

pub const QUIC_PORT: u16 = 50000;
pub const TCP_PORT: u16 = 50001;

pub const DEFAULT_CHUNK_SIZE: usize: 1024 * 1024;
pub const MAX_CHUNK_SIZE: usize = 4 * 1024 * 1024;

pub const SESSION_KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

pub const CONNECTION_TIMEOUT_SECS: u64 = 10;
pub const TRANSFER_TIMEOUT_SECS: u64 = 60;