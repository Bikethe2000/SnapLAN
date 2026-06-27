use thiserror::Error;

#[derive(Error, Debug)]
pub enum SnapError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Peer Not Found")]
    PeerNotFound,
}