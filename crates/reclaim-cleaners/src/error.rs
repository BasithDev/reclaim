//! Error type shared by every reclaim-cleaners module.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CleanError {
    #[error("unknown cleaner: {0}")]
    UnknownCleaner(String),
    #[error("refused to touch protected path: {0}")]
    Protected(String),
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("command `{command}` failed: {message}")]
    Command { command: String, message: String },
    #[error("administrator permission was denied or cancelled")]
    ElevationDenied,
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CleanError>;
