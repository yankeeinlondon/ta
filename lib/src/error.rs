use thiserror::Error;

/// Main error type for the TypeScript Analyzer library.
///
/// This enum encapsulates all possible errors that can occur during
/// the parsing, semantic analysis, and data extraction phases of the
/// library's operation.
#[derive(Debug, Error)]
pub enum Error {
    /// Represents an error that occurred during the parsing of a TypeScript file.
    ///
    /// This error includes the file path where the error occurred and a
    /// descriptive message from the parser.
    #[error("Parse error in {file}: {message}")]
    ParseError {
        /// The path to the file where the parse error occurred.
        file: String,
        /// The error message provided by the parser.
        message: String,
    },

    /// Represents an error that occurred during the semantic analysis phase.
    ///
    /// Semantic errors typically involve issues with symbol resolution,
    /// scope analysis, or other high-level language constructs.
    #[error("Semantic analysis failed: {0}")]
    SemanticError(String),

    /// Represents a standard I/O error.
    ///
    /// This variant wraps `std::io::Error` and occurs when reading files,
    /// writing output, or interacting with the filesystem.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Represents an error where the source type could not be determined or is unsupported.
    ///
    /// This typically happens when a file extension is not recognized as
    /// a valid TypeScript or JavaScript file.
    #[error("Invalid source type for {0}")]
    InvalidSourceType(String),

    /// Represents a general analysis error that doesn't fit into other categories.
    ///
    /// This can be used for logical errors during the analysis pipeline.
    #[error("Analysis error: {0}")]
    AnalysisError(String),
}