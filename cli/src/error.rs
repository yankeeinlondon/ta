use thiserror::Error;
use ta_lib::Error as LibError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Analysis failed: {0}")]
    Analysis(#[from] LibError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
}
