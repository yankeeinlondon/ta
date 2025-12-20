use thiserror::Error;
use ta_lib::Error as LibError;

/// Main error type for the TypeScript Analyzer CLI.
///
/// This enum handles errors specific to the command-line interface,
/// such as configuration issues, argument parsing errors, and
/// errors propagated from the underlying library.
#[derive(Debug, Error)]
pub enum Error {
    /// Represents an error in the CLI configuration.
    ///
    /// This can occur when config files are malformed or missing required fields.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Represents an error propagated from the `ta-lib` library.
    ///
    /// This variant wraps the library's error type, allowing the CLI
    /// to report core analysis failures.
    #[error("Analysis failed: {0}")]
    Analysis(#[from] LibError),

    /// Represents a standard I/O error occurring within the CLI layer.
    ///
    /// This allows the CLI to handle file operations (like reading config)
    /// distinct from the library's I/O operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Represents an error due to invalid command-line arguments.
    ///
    /// While `clap` handles most argument parsing, this variant is for
    /// logic errors involving valid but conflicting or nonsensical arguments.
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
}