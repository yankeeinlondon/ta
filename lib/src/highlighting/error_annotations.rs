/// Error annotation module for TypeScript Analyzer highlighting.
///
/// This module provides error annotation capabilities for code blocks,
/// including "red squigglies" visual indicators and error messages.
/// Uses `Span` as the single source of truth for error positions.

use oxc_span::Span;
use serde::Serialize;
use std::collections::HashMap;

use crate::highlighting::ansi::AnsiBuilder;

/// Severity level for error annotations.
///
/// This enum is marked `#[non_exhaustive]` to allow future additions
/// without breaking changes.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::error_annotations::ErrorSeverity;
///
/// let severity = ErrorSeverity::Error;
/// assert_eq!(severity, ErrorSeverity::Error);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    /// Critical error that prevents compilation.
    Error,
    /// Warning that should be addressed but doesn't prevent compilation.
    Warning,
    /// Informational message or suggestion.
    Info,
}

impl ErrorSeverity {
    /// Returns the RGB color for this severity level.
    ///
    /// - Error: Red (255, 0, 0)
    /// - Warning: Orange (255, 165, 0)
    /// - Info: Blue (0, 150, 255)
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            ErrorSeverity::Error => (255, 0, 0),
            ErrorSeverity::Warning => (255, 165, 0),
            ErrorSeverity::Info => (0, 150, 255),
        }
    }

    /// Returns the CSS class name for this severity level.
    pub fn css_class(&self) -> &'static str {
        match self {
            ErrorSeverity::Error => "error",
            ErrorSeverity::Warning => "warning",
            ErrorSeverity::Info => "info",
        }
    }
}

/// Error annotation with position and message.
///
/// Uses `Span` as the single source of truth for error positions.
/// Line and column positions are computed on-demand from the span
/// and source text.
///
/// # Examples
///
/// ```
/// use oxc_span::Span;
/// use ta_lib::highlighting::error_annotations::{ErrorAnnotation, ErrorSeverity};
///
/// let span = Span::new(10, 15);
/// let annotation = ErrorAnnotation::new(
///     span,
///     "Type 'string' is not assignable to type 'number'".to_string(),
///     ErrorSeverity::Error,
/// );
///
/// assert_eq!(annotation.message(), "Type 'string' is not assignable to type 'number'");
/// assert_eq!(annotation.severity(), ErrorSeverity::Error);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ErrorAnnotation {
    /// The span indicating the error location (private - single source of truth).
    #[serde(skip)]
    span: Span,
    /// The error message.
    message: String,
    /// The severity level.
    severity: ErrorSeverity,
}

impl ErrorAnnotation {
    /// Creates a new error annotation.
    ///
    /// # Arguments
    ///
    /// * `span` - The byte range of the error in the source text
    /// * `message` - The error message to display
    /// * `severity` - The severity level of the error
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_span::Span;
    /// use ta_lib::highlighting::error_annotations::{ErrorAnnotation, ErrorSeverity};
    ///
    /// let annotation = ErrorAnnotation::new(
    ///     Span::new(0, 5),
    ///     "Unused variable".to_string(),
    ///     ErrorSeverity::Warning,
    /// );
    /// ```
    pub fn new(span: Span, message: String, severity: ErrorSeverity) -> Self {
        Self {
            span,
            message,
            severity,
        }
    }

    /// Returns the span for this error.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the severity level.
    pub fn severity(&self) -> ErrorSeverity {
        self.severity
    }

    /// Computes the 1-based line number where the error starts.
    ///
    /// This is computed from the span's start position by counting newlines
    /// in the source text before that position.
    ///
    /// # Arguments
    ///
    /// * `source` - The full source text
    ///
    /// # Returns
    ///
    /// The line number (1-based) where the error starts.
    ///
    /// # Panics
    ///
    /// Panics if the span is out of bounds for the source text.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_span::Span;
    /// use ta_lib::highlighting::error_annotations::{ErrorAnnotation, ErrorSeverity};
    ///
    /// let source = "line 1\nline 2\nline 3";
    /// let annotation = ErrorAnnotation::new(
    ///     Span::new(7, 12), // "line 2"
    ///     "Error on line 2".to_string(),
    ///     ErrorSeverity::Error,
    /// );
    ///
    /// assert_eq!(annotation.line(source), 2);
    /// ```
    pub fn line(&self, source: &str) -> usize {
        let start = self.span.start as usize;
        if start > source.len() {
            panic!(
                "Span start {} is out of bounds for source length {}",
                start,
                source.len()
            );
        }
        source[..start].chars().filter(|&c| c == '\n').count() + 1
    }

    /// Computes the 1-based column number where the error starts.
    ///
    /// This is computed from the span's start position by finding the last
    /// newline before it and counting characters from there.
    ///
    /// # Arguments
    ///
    /// * `source` - The full source text
    ///
    /// # Returns
    ///
    /// The column number (1-based) where the error starts.
    ///
    /// # Panics
    ///
    /// Panics if the span is out of bounds for the source text.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_span::Span;
    /// use ta_lib::highlighting::error_annotations::{ErrorAnnotation, ErrorSeverity};
    ///
    /// let source = "const x = 42;";
    /// let annotation = ErrorAnnotation::new(
    ///     Span::new(6, 7), // "x"
    ///     "Unused variable 'x'".to_string(),
    ///     ErrorSeverity::Warning,
    /// );
    ///
    /// assert_eq!(annotation.column(source), 7);
    /// ```
    pub fn column(&self, source: &str) -> usize {
        let start = self.span.start as usize;
        if start > source.len() {
            panic!(
                "Span start {} is out of bounds for source length {}",
                start,
                source.len()
            );
        }
        let line_start = source[..start]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        source[line_start..start].chars().count() + 1
    }

    /// Computes the 1-based line number where the error ends.
    ///
    /// This is computed from the span's end position by counting newlines
    /// in the source text before that position.
    ///
    /// # Arguments
    ///
    /// * `source` - The full source text
    ///
    /// # Returns
    ///
    /// The line number (1-based) where the error ends.
    ///
    /// # Panics
    ///
    /// Panics if the span is out of bounds for the source text.
    pub fn end_line(&self, source: &str) -> usize {
        let end = self.span.end as usize;
        if end > source.len() {
            panic!(
                "Span end {} is out of bounds for source length {}",
                end,
                source.len()
            );
        }
        source[..end].chars().filter(|&c| c == '\n').count() + 1
    }

    /// Computes the 1-based column number where the error ends.
    ///
    /// This is computed from the span's end position by finding the last
    /// newline before it and counting characters from there.
    ///
    /// # Arguments
    ///
    /// * `source` - The full source text
    ///
    /// # Returns
    ///
    /// The column number (1-based) where the error ends.
    ///
    /// # Panics
    ///
    /// Panics if the span is out of bounds for the source text.
    pub fn end_column(&self, source: &str) -> usize {
        let end = self.span.end as usize;
        if end > source.len() {
            panic!(
                "Span end {} is out of bounds for source length {}",
                end,
                source.len()
            );
        }
        let line_start = source[..end].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
        source[line_start..end].chars().count() + 1
    }

    /// Renders this error annotation for console output with ANSI escape codes.
    ///
    /// Creates a visual representation with:
    /// - The line of code containing the error
    /// - A red underline beneath the error span
    /// - The error message below the underline
    ///
    /// # Arguments
    ///
    /// * `source` - The full source text
    ///
    /// # Returns
    ///
    /// A string with ANSI escape codes for terminal display.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_span::Span;
    /// use ta_lib::highlighting::error_annotations::{ErrorAnnotation, ErrorSeverity};
    ///
    /// let source = "const x = 'hello';";
    /// let annotation = ErrorAnnotation::new(
    ///     Span::new(10, 17), // 'hello'
    ///     "Type 'string' is not assignable to type 'number'".to_string(),
    ///     ErrorSeverity::Error,
    /// );
    ///
    /// let output = annotation.render_console(source);
    /// assert!(output.contains("const x = 'hello';"));
    /// assert!(output.contains("\x1b[")); // ANSI escape code
    /// ```
    pub fn render_console(&self, source: &str) -> String {
        let line_num = self.line(source);
        let col = self.column(source);
        let end_col = self.end_column(source);

        // Extract the line containing the error
        let lines: Vec<&str> = source.lines().collect();
        let error_line = if line_num > 0 && line_num <= lines.len() {
            lines[line_num - 1]
        } else {
            ""
        };

        // Build the underline (red squiggly)
        let (r, g, b) = self.severity.color();
        let underline_code = AnsiBuilder::new().fg_rgb(r, g, b).underline().build();

        // Calculate the span of the underline
        let underline_start = col - 1;
        let underline_length = if line_num == self.end_line(source) {
            end_col - col
        } else {
            error_line.chars().count() - underline_start
        };

        // Build the underline string
        let mut underline = String::new();
        for _ in 0..underline_start {
            underline.push(' ');
        }
        underline.push_str(&underline_code);
        for _ in 0..underline_length.max(1) {
            underline.push('^');
        }
        underline.push_str(AnsiBuilder::RESET);

        format!(
            "{}\n{}\n{}{}\n",
            error_line, underline, underline_code, self.message
        )
    }

    /// Renders this error annotation for HTML output with popover API.
    ///
    /// Creates semantic HTML with:
    /// - A `<span>` element wrapping the error text
    /// - ARIA attributes for accessibility
    /// - A popover div with the error message
    ///
    /// # Arguments
    ///
    /// * `source` - The full source text
    /// * `error_id` - A unique ID for this error (used for popover linking)
    ///
    /// # Returns
    ///
    /// An HTML string with popover markup and ARIA attributes.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_span::Span;
    /// use ta_lib::highlighting::error_annotations::{ErrorAnnotation, ErrorSeverity};
    ///
    /// let source = "const x = 'hello';";
    /// let annotation = ErrorAnnotation::new(
    ///     Span::new(10, 17),
    ///     "Type error".to_string(),
    ///     ErrorSeverity::Error,
    /// );
    ///
    /// let html = annotation.render_html(source, 1);
    /// assert!(html.contains("popovertarget"));
    /// assert!(html.contains("aria-describedby"));
    /// ```
    pub fn render_html(&self, source: &str, error_id: usize) -> String {
        let start = self.span.start as usize;
        let end = self.span.end as usize;

        if start > source.len() || end > source.len() {
            return String::new();
        }

        let error_text = &source[start..end];
        let severity_class = self.severity.css_class();
        let popover_id = format!("error-{}", error_id);

        format!(
            r#"<span class="error-highlight {}" popovertarget="{}" aria-describedby="{}">
  <span class="squiggle" aria-label="{}">{}</span>
</span>
<div id="{}" popover role="alert">
  <div class="error-message">{}</div>
</div>"#,
            severity_class,
            popover_id,
            popover_id,
            self.severity.css_class(),
            html_escape::encode_text(error_text),
            popover_id,
            html_escape::encode_text(&self.message)
        )
    }
}

/// Renders multiple error annotations for console output.
///
/// Handles overlapping spans gracefully by rendering each error
/// on its own line.
///
/// # Arguments
///
/// * `source` - The full source text
/// * `annotations` - A slice of error annotations to render
///
/// # Returns
///
/// A string with ANSI escape codes showing all errors.
///
/// # Examples
///
/// ```
/// use oxc_span::Span;
/// use ta_lib::highlighting::error_annotations::{ErrorAnnotation, ErrorSeverity, render_errors_console};
///
/// let source = "const x = 'hello';\nconst y = 42;";
/// let errors = vec![
///     ErrorAnnotation::new(Span::new(10, 17), "Error 1".to_string(), ErrorSeverity::Error),
///     ErrorAnnotation::new(Span::new(28, 30), "Error 2".to_string(), ErrorSeverity::Warning),
/// ];
///
/// let output = render_errors_console(source, &errors);
/// assert!(output.contains("Error 1"));
/// assert!(output.contains("Error 2"));
/// ```
pub fn render_errors_console(source: &str, annotations: &[ErrorAnnotation]) -> String {
    let mut output = String::new();

    for annotation in annotations {
        output.push_str(&annotation.render_console(source));
        output.push('\n');
    }

    output
}

/// Renders multiple error annotations for HTML output.
///
/// Creates a map of error IDs to HTML fragments that can be
/// inserted into the highlighted code.
///
/// # Arguments
///
/// * `source` - The full source text
/// * `annotations` - A slice of error annotations to render
///
/// # Returns
///
/// A HashMap mapping error IDs to HTML strings.
///
/// # Examples
///
/// ```
/// use oxc_span::Span;
/// use ta_lib::highlighting::error_annotations::{ErrorAnnotation, ErrorSeverity, render_errors_html};
///
/// let source = "const x = 'hello';";
/// let errors = vec![
///     ErrorAnnotation::new(Span::new(10, 17), "Error".to_string(), ErrorSeverity::Error),
/// ];
///
/// let html_map = render_errors_html(source, &errors);
/// assert_eq!(html_map.len(), 1);
/// ```
pub fn render_errors_html(
    source: &str,
    annotations: &[ErrorAnnotation],
) -> HashMap<usize, String> {
    let mut html_map = HashMap::new();

    for (idx, annotation) in annotations.iter().enumerate() {
        let error_id = idx + 1;
        html_map.insert(error_id, annotation.render_html(source, error_id));
    }

    html_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_color() {
        assert_eq!(ErrorSeverity::Error.color(), (255, 0, 0));
        assert_eq!(ErrorSeverity::Warning.color(), (255, 165, 0));
        assert_eq!(ErrorSeverity::Info.color(), (0, 150, 255));
    }

    #[test]
    fn test_error_severity_css_class() {
        assert_eq!(ErrorSeverity::Error.css_class(), "error");
        assert_eq!(ErrorSeverity::Warning.css_class(), "warning");
        assert_eq!(ErrorSeverity::Info.css_class(), "info");
    }

    #[test]
    fn test_error_annotation_new() {
        let span = Span::new(10, 15);
        let annotation = ErrorAnnotation::new(
            span,
            "Test error".to_string(),
            ErrorSeverity::Error,
        );

        assert_eq!(annotation.span(), span);
        assert_eq!(annotation.message(), "Test error");
        assert_eq!(annotation.severity(), ErrorSeverity::Error);
    }

    #[test]
    fn test_line_computation_first_line() {
        let source = "const x = 42;";
        let annotation = ErrorAnnotation::new(
            Span::new(6, 7),
            "Error".to_string(),
            ErrorSeverity::Error,
        );

        assert_eq!(annotation.line(source), 1);
    }

    #[test]
    fn test_line_computation_second_line() {
        let source = "line 1\nline 2\nline 3";
        let annotation = ErrorAnnotation::new(
            Span::new(7, 12), // "line 2"
            "Error".to_string(),
            ErrorSeverity::Error,
        );

        assert_eq!(annotation.line(source), 2);
    }

    #[test]
    fn test_column_computation() {
        let source = "const x = 42;";
        let annotation = ErrorAnnotation::new(
            Span::new(6, 7), // "x"
            "Error".to_string(),
            ErrorSeverity::Error,
        );

        assert_eq!(annotation.column(source), 7);
    }

    #[test]
    fn test_end_line_computation() {
        let source = "line 1\nline 2";
        let annotation = ErrorAnnotation::new(
            Span::new(0, 13), // Spans both lines
            "Error".to_string(),
            ErrorSeverity::Error,
        );

        assert_eq!(annotation.end_line(source), 2);
    }

    #[test]
    fn test_end_column_computation() {
        let source = "const x = 42;";
        let annotation = ErrorAnnotation::new(
            Span::new(6, 13), // "x = 42;"
            "Error".to_string(),
            ErrorSeverity::Error,
        );

        assert_eq!(annotation.end_column(source), 14);
    }

    #[test]
    fn test_console_rendering_contains_ansi_codes() {
        let source = "const x = 'hello';";
        let annotation = ErrorAnnotation::new(
            Span::new(10, 17),
            "Type error".to_string(),
            ErrorSeverity::Error,
        );

        let output = annotation.render_console(source);
        assert!(output.contains("\x1b[")); // ANSI escape code
        assert!(output.contains("Type error"));
        assert!(output.contains("const x = 'hello';"));
    }

    #[test]
    fn test_console_rendering_has_underline() {
        let source = "const x = 42;";
        let annotation = ErrorAnnotation::new(
            Span::new(10, 12),
            "Error".to_string(),
            ErrorSeverity::Error,
        );

        let output = annotation.render_console(source);
        assert!(output.contains("^")); // Underline character
    }

    #[test]
    fn test_html_rendering_has_popover() {
        let source = "const x = 'hello';";
        let annotation = ErrorAnnotation::new(
            Span::new(10, 17),
            "Type error".to_string(),
            ErrorSeverity::Error,
        );

        let html = annotation.render_html(source, 1);
        assert!(html.contains("popovertarget=\"error-1\""));
        assert!(html.contains("aria-describedby=\"error-1\""));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("Type error"));
    }

    #[test]
    fn test_html_rendering_has_aria_labels() {
        let source = "const x = 42;";
        let annotation = ErrorAnnotation::new(
            Span::new(6, 7),
            "Unused".to_string(),
            ErrorSeverity::Warning,
        );

        let html = annotation.render_html(source, 1);
        assert!(html.contains("aria-label=\"warning\""));
    }

    #[test]
    fn test_render_multiple_errors_console() {
        let source = "const x = 'hello';\nconst y = 42;";
        let errors = vec![
            ErrorAnnotation::new(Span::new(10, 17), "Error 1".to_string(), ErrorSeverity::Error),
            ErrorAnnotation::new(Span::new(28, 30), "Error 2".to_string(), ErrorSeverity::Warning),
        ];

        let output = render_errors_console(source, &errors);
        assert!(output.contains("Error 1"));
        assert!(output.contains("Error 2"));
    }

    #[test]
    fn test_render_multiple_errors_html() {
        let source = "const x = 'hello';";
        let errors = vec![
            ErrorAnnotation::new(Span::new(6, 7), "Error 1".to_string(), ErrorSeverity::Error),
            ErrorAnnotation::new(Span::new(10, 17), "Error 2".to_string(), ErrorSeverity::Warning),
        ];

        let html_map = render_errors_html(source, &errors);
        assert_eq!(html_map.len(), 2);
        assert!(html_map.contains_key(&1));
        assert!(html_map.contains_key(&2));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_line_panics_on_out_of_bounds() {
        let source = "short";
        let annotation = ErrorAnnotation::new(
            Span::new(100, 105),
            "Error".to_string(),
            ErrorSeverity::Error,
        );

        let _ = annotation.line(source);
    }

    #[test]
    fn test_overlapping_spans_handled() {
        let source = "const x = 42;";
        let errors = vec![
            ErrorAnnotation::new(Span::new(6, 12), "Error 1".to_string(), ErrorSeverity::Error),
            ErrorAnnotation::new(Span::new(10, 12), "Error 2".to_string(), ErrorSeverity::Warning),
        ];

        let output = render_errors_console(source, &errors);
        assert!(output.contains("Error 1"));
        assert!(output.contains("Error 2"));
    }

    #[test]
    fn test_error_annotation_is_serializable() {
        let annotation = ErrorAnnotation::new(
            Span::new(0, 5),
            "Test".to_string(),
            ErrorSeverity::Error,
        );

        let json = serde_json::to_string(&annotation);
        assert!(json.is_ok());
    }

    #[test]
    fn test_error_severity_is_serializable() {
        let severity = ErrorSeverity::Warning;
        let json = serde_json::to_string(&severity);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), "\"warning\"");
    }
}
