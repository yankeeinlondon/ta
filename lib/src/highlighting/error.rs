use thiserror::Error;

/// Errors that can occur during code highlighting operations.
///
/// This error type uses `thiserror` for ergonomic error handling and provides
/// detailed error messages for all highlighting failures.
#[derive(Error, Debug)]
pub enum HighlightError {
    /// The specified language is not supported by the syntax highlighter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::error::HighlightError;
    /// let error = HighlightError::UnsupportedLanguage("cobol".to_string());
    /// assert_eq!(error.to_string(), "Unsupported language: cobol");
    /// ```
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// The requested theme was not found in the available themes.
    ///
    /// This can occur when requesting a built-in theme that doesn't exist
    /// or when attempting to load a custom theme file that cannot be found.
    #[error("Theme '{name}' not found")]
    ThemeNotFound { name: String },

    /// Failed to load a theme from a file.
    ///
    /// This typically indicates an I/O error (file not found, permissions)
    /// or a malformed theme file.
    #[error("Failed to load theme from file: {source}")]
    ThemeLoadError {
        #[from]
        source: std::io::Error,
    },

    /// The code span is invalid (out of bounds or malformed).
    ///
    /// This error is returned when trying to highlight a span that extends
    /// beyond the source code boundaries.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::error::HighlightError;
    /// let error = HighlightError::InvalidSpan { line: 100, column: 50 };
    /// assert_eq!(error.to_string(), "Invalid code span: line 100, column 50");
    /// ```
    #[error("Invalid code span: line {line}, column {column}")]
    InvalidSpan { line: usize, column: usize },

    /// The code block exceeds the maximum allowed size.
    ///
    /// This limit exists to prevent excessive memory usage and performance
    /// degradation when highlighting very large files.
    #[error("Code block exceeds maximum size ({size} lines > {max} lines)")]
    CodeBlockTooLarge { size: usize, max: usize },

    /// An internal error occurred in the syntax highlighting engine.
    ///
    /// This is a catch-all for unexpected syntect errors.
    #[error("Syntax highlighting failed: {0}")]
    SyntectError(String),
}

/// A specialized `Result` type for highlighting operations.
///
/// This is a convenience alias for `std::result::Result<T, HighlightError>`.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::error::Result;
///
/// fn highlight_something() -> Result<String> {
///     Ok("highlighted code".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, HighlightError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_language_error() {
        let error = HighlightError::UnsupportedLanguage("cobol".to_string());
        assert_eq!(error.to_string(), "Unsupported language: cobol");
    }

    #[test]
    fn test_theme_not_found_error() {
        let error = HighlightError::ThemeNotFound {
            name: "NonExistent".to_string(),
        };
        assert_eq!(error.to_string(), "Theme 'NonExistent' not found");
    }

    #[test]
    fn test_invalid_span_error() {
        let error = HighlightError::InvalidSpan {
            line: 100,
            column: 50,
        };
        assert_eq!(error.to_string(), "Invalid code span: line 100, column 50");
    }

    #[test]
    fn test_code_block_too_large_error() {
        let error = HighlightError::CodeBlockTooLarge {
            size: 15000,
            max: 10000,
        };
        assert_eq!(
            error.to_string(),
            "Code block exceeds maximum size (15000 lines > 10000 lines)"
        );
    }

    #[test]
    fn test_syntect_error() {
        let error = HighlightError::SyntectError("parse failed".to_string());
        assert_eq!(error.to_string(), "Syntax highlighting failed: parse failed");
    }

    #[test]
    fn test_theme_load_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = HighlightError::ThemeLoadError { source: io_error };
        assert!(error.to_string().contains("Failed to load theme from file"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        assert_eq!(returns_result().unwrap(), 42);
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<HighlightError>();
        assert_sync::<HighlightError>();
    }
}
