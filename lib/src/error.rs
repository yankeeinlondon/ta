use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Parse error in {file}: {message}")]
    ParseError { file: String, message: String },

    #[error("Semantic analysis failed: {0}")]
    SemanticError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid source type for {0}")]
    InvalidSourceType(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),
}
