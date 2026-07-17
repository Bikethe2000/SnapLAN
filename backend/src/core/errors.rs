use thiserror::Error;

pub type Result<T> = std::result::Result<T, SnapError>;

#[derive(Debug, Error)]
pub enum SnapError. {
    #[error("Invalid state")]
    InvalidState,

    #[error("Invalid Identity")]
    InvalidIdentity,

    #[error("Peer not found")]
    PeerNotFound,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Pairing error: {0}")]
    Pairing(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}