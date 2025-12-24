/// Code highlighting and formatting module for TypeScript Analyzer.
///
/// This module provides syntax highlighting capabilities for code blocks,
/// context-aware code extraction, and error annotation rendering.
// Phase 1: Core Highlighting Infrastructure
pub mod ansi;
pub mod error;
pub mod options;
pub mod syntect_highlighter;
pub mod themes;

// Phase 2: Error Annotation System
pub mod error_annotations;

// Phase 3: Context-Aware Code Extraction
pub mod code_context;

// Phase 4: Markdown Parsing with Code Blocks
pub mod markdown_formatter;

// Re-export commonly used types from Phase 1
pub use error::{HighlightError, Result};
pub use options::{HighlightOptions, MarkdownOptions};
pub use syntect_highlighter::{highlight_code, HighlightedCode, HighlightSegment, RgbColor, SegmentStyle};
pub use themes::{BuiltinTheme, ThemeSource};

// Re-export Phase 2 types
pub use error_annotations::{ErrorAnnotation, ErrorSeverity, render_errors_console, render_errors_html};

// Re-export Phase 3 types
pub use code_context::{extract_code_context, CodeContext, ScopeType, TruncationInfo};

// Re-export Phase 4 types
pub use markdown_formatter::{format_markdown, parse_code_block_info, FormattedMarkdown};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_code_api() {
        let code = "function test() { return 42; }";
        let options = HighlightOptions::new("js");

        let result = highlight_code(code, options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_markdown_implemented() {
        let markdown = "# Hello\n\n```ts\nconst x = 42;\n```";
        let options = MarkdownOptions::default();

        let result = format_markdown(markdown, options);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted.code_block_count, 1);
    }

    #[test]
    fn test_module_exports() {
        // Verify that all exported types are accessible
        let _options = HighlightOptions::default();
        let _md_options = MarkdownOptions::default();
        let _theme = BuiltinTheme::SolarizedLight;
    }
}
