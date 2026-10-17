//! Error type shared by every reclaim-core module.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("path not found: {0}")]
    NotFound(String),
    #[error("scan cancelled")]
    Cancelled,
    #[error("snapshot store error: {0}")]
    Store(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
