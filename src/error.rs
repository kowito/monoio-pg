use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Postgres protocol error: {0}")]
    Protocol(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Connection closed")]
    Closed,

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
