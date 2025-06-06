use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HbpError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON parse error: {0}")]
    Parse(String),

    #[error("Hetzner action failed: {0}")]
    ActionFailed(String),

    #[error("CLI arg / env error: {0}")]
    Cli(String),

    #[error("Other: {0}")]
    Other(String),
}

impl From<serde_json::Error> for HbpError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parse(err.to_string())
    }
}

impl From<hex::FromHexError> for HbpError {
    fn from(err: hex::FromHexError) -> Self {
        Self::Parse(err.to_string())
    }
}

impl From<std::num::ParseIntError> for HbpError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::Parse(err.to_string())
    }
}
