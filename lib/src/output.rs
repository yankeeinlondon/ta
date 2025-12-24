use crate::models::{SymbolInfo, TypeError};
use crate::highlighting::{highlight_code, HighlightOptions};
use serde::Serialize;
use clap::ValueEnum;
use colored::*;
use std::path::Path;

/// Create a clickable terminal link using OSC8 standard
///
/// The displayed text remains a relative path, but the link target is an absolute path.
/// This allows terminals that support OSC8 to make file paths clickable.
///
/// Format: \x1b]8;;file://absolute_path\x1b\\display_text\x1b]8;;\x1b\\
pub fn link_file(text: &str, filepath: &str) -> String {
    // Convert to absolute path
    let abs_path = if Path::new(filepath).is_absolute() {
        filepath.to_string()
    } else {
        // Get current directory and join with relative path
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(filepath).to_string_lossy().to_string(),
            Err(_) => filepath.to_string(), // Fallback to relative if current_dir fails
        }
    };

    // OSC8 format: ESC]8;;URI ESC\\ TEXT ESC]8;; ESC\\
    format!("\x1b]8;;file://{}\x1b\\{}\x1b]8;;\x1b\\", abs_path, text)
}

#[derive(Debug, Clone, Copy, ValueEnum, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Console,
    Html,
    Json,
}

pub struct OutputFormatter;

impl OutputFormatter {
    /// Format a symbol signature with colors for console output
    pub fn format_symbol_signature_colored(symbol: &SymbolInfo) -> String {
        use colored::*;

        match symbol.kind {
            crate::models::SymbolKind::Function => {
                let keyword = "function".magenta();
                let name = symbol.name.cyan().bold();

                let params = if let Some(params) = &symbol.parameters {
                    params.iter()
                        .map(|p| {
                            if let Some(ty) = &p.type_annotation {
                                format!("{}: {}", p.name.yellow(), ty.green())
                            } else {
                                p.name.yellow().to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    String::new()
                };

                if let Some(ret) = &symbol.return_type {
                    format!("{} {}({}): {}", keyword, name, params, ret.green())
                } else {
                    format!("{} {}({})", keyword, name, params)
                }
            }
            crate::models::SymbolKind::Class => {
                format!("{} {}", "class".magenta(), symbol.name.cyan().bold())
            }
            crate::models::SymbolKind::Interface => {
                let keyword = "interface".magenta();
                let name = symbol.name.cyan().bold();

                if let Some(props) = &symbol.properties {
                    if props.is_empty() {
                        format!("{} {}", keyword, name)
                    } else {
                        let prop_str = props.iter()
                            .take(3)
                            .map(|p| {
                                if let Some(ty) = &p.type_annotation {
                                    format!("{}: {}", p.name.yellow(), ty.green())
                                } else {
                                    p.name.yellow().to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ");

                        let suffix = if props.len() > 3 { ", ..." } else { "" };
                        format!("{} {} {{ {}{} }}", keyword, name, prop_str, suffix)
                    }
                } else {
                    format!("{} {}", keyword, name)
                }
            }
            crate::models::SymbolKind::Type => {
                format!("{} {}", "type".magenta(), symbol.name.cyan().bold())
            }
            crate::models::SymbolKind::Variable => {
                format!("{} {}", "variable".magenta(), symbol.name.cyan().bold())
            }
            crate::models::SymbolKind::Enum => {
                format!("{} {}", "enum".magenta(), symbol.name.cyan().bold())
            }
        }
    }

    pub fn format_type_errors(errors: &[TypeError], format: OutputFormat) -> String {
        match format {
            OutputFormat::Console => Self::format_type_errors_console(errors),
            OutputFormat::Html => Self::format_type_errors_html(errors),
            OutputFormat::Json => serde_json::to_string_pretty(errors).unwrap_or_default(),
        }
    }

    pub fn format_symbols(symbols: &[SymbolInfo], format: OutputFormat) -> String {
        match format {
            OutputFormat::Console => Self::format_symbols_console(symbols),
            OutputFormat::Html => Self::format_symbols_html(symbols),
            OutputFormat::Json => serde_json::to_string_pretty(symbols).unwrap_or_default(),
        }
    }

    fn format_type_errors_console(errors: &[TypeError]) -> String {
        let mut output = String::new();

        for error in errors {
            // New format: [❌] Message (bold)
            //   in scope at file:line:col
            output.push_str(&format!(
                "{} {}\n",
                "[❌]".red().bold(),
                error.message.bold()
            ));

            // Location line: in scope at file:line:col
            // Use OSC8 hyperlink for clickable file path
            let file_with_location = format!("{}:{}:{}", error.file, error.line, error.column);
            let linked_file = link_file(&file_with_location, &error.file).blue();

            output.push_str(&format!(
                "  {} {} {} {}\n\n",  // Add blank line after location
                "in".dimmed(),
                error.scope.cyan(),
                "at".dimmed(),
                linked_file
            ));

            // Use new highlighting if available, fallback to legacy
            if let Some(source) = &error.source_code {
                // Create highlighting options with error annotations
                // Note: TypeScript is a superset of JavaScript, so we use "js" syntax
                // which is what syntect supports (TypeScript syntax is not included)
                let options = HighlightOptions::new("js")
                    .with_line_numbers(true)
                    .with_indent(2)  // Indent code blocks for visual nesting
                    .for_format(OutputFormat::Console);

                match highlight_code(&source.display_code, options) {
                    Ok(highlighted) => {
                        output.push_str(&highlighted.render_console());
                        output.push('\n');
                    }
                    Err(e) => {
                        // Log error for debugging, fallback to plain text
                        log::debug!("Highlighting failed: {}", e);
                        output.push_str(&format!("  {}\n", source.display_code.dimmed()));
                    }
                }
            } else if !error.block.is_empty() {
                // Legacy fallback
                output.push_str(&format!("  {}\n", error.block.dimmed()));
            }

            output.push('\n');
        }

        output
    }

    fn format_type_errors_html(errors: &[TypeError]) -> String {
        let mut output = String::from("<div class=\"type-errors\">\n");

        for error in errors {
            output.push_str(&format!(
                r#"<div class="error-block">
  <div class="error-header">
    <span class="error-id">[{}]</span>
    <span class="keyword">in</span>
    <span class="scope">{}</span>
  </div>
  <div class="error-location">
    <span class="keyword">at</span>
    <span class="file-path">{}:{}:{}</span>
  </div>
  <div class="error-message">{}</div>
"#,
                html_escape::encode_text(&error.id),
                html_escape::encode_text(&error.scope),
                html_escape::encode_text(&error.file),
                error.line,
                error.column,
                html_escape::encode_text(&error.message)
            ));

            // Use highlighting for HTML output
            // TypeScript uses JavaScript syntax (syntect doesn't have native TS support)
            if let Some(source) = &error.source_code {
                let options = HighlightOptions::new("js")
                    .with_line_numbers(true)
                    .with_indent(2)  // Indent code blocks for visual nesting
                    .for_format(OutputFormat::Html);

                match highlight_code(&source.display_code, options) {
                    Ok(highlighted) => {
                        output.push_str("  <div class=\"code-highlight\">\n");
                        output.push_str(&highlighted.render_html());
                        output.push_str("  </div>\n");
                    }
                    Err(_) => {
                        // Fallback
                        output.push_str(&format!("  <pre>{}</pre>\n",
                            html_escape::encode_text(&source.display_code)));
                    }
                }
            } else if !error.block.is_empty() {
                // Legacy fallback
                output.push_str(&format!("  <pre>{}</pre>\n",
                    html_escape::encode_text(&error.block)));
            }

            output.push_str("</div>\n");
        }

        output.push_str("</div>");
        output
    }

    fn format_symbols_console(symbols: &[SymbolInfo]) -> String {
        let mut output = String::new();

        for symbol in symbols {
            // Use colored signature display
            let signature = Self::format_symbol_signature_colored(symbol);

            // File path in blue
            let file_str = format!("{}:{}-{}", symbol.file, symbol.start_line, symbol.end_line).blue();

            output.push_str(&format!("{} {}\n", signature, file_str));

            // JSDoc if present
            if let Some(jsdoc) = &symbol.jsdoc {
                output.push_str(&format!("  {}\n", jsdoc.dimmed().italic()));
            }

            output.push('\n');
        }

        output
    }

    fn format_symbols_html(symbols: &[SymbolInfo]) -> String {
        let mut output = String::from("<div class=\"symbols\">\n");

        for symbol in symbols {
            output.push_str(&format!(
                r#"<div class="symbol-block" data-kind="{:?}">
  <div class="symbol-header">
    <span class="keyword">{:?}</span>
    <span class="symbol-name">{}</span>
    <span class="file-path">{}:{}-{}</span>
  </div>
"#,
                symbol.kind,
                symbol.kind,
                html_escape::encode_text(&symbol.name),
                html_escape::encode_text(&symbol.file),
                symbol.start_line,
                symbol.end_line
            ));

            // Parameters
            if let Some(params) = &symbol.parameters {
                if !params.is_empty() {
                    output.push_str(r#"  <div class="parameters">"#);
                    for param in params {
                        output.push_str(&format!(
                            r#"<span class="type-annotation">{}</span>"#,
                            html_escape::encode_text(&param.name)
                        ));
                    }
                    output.push_str("</div>\n");
                }
            }

            // Properties
            if let Some(props) = &symbol.properties {
                if !props.is_empty() {
                    output.push_str(r#"  <div class="properties">"#);
                    for prop in props {
                        output.push_str(&format!(
                            r#"<span class="type-annotation">{}</span>"#,
                            html_escape::encode_text(&prop.name)
                        ));
                    }
                    output.push_str("</div>\n");
                }
            }

            output.push_str("</div>\n");
        }

        output.push_str("</div>");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_span::Span;
    use crate::models::SymbolKind;
    use serial_test::serial;

    #[test]
    fn test_format_type_errors_json() {
        let error = TypeError {
            id: "TS100".to_string(),
            message: "Error msg".to_string(),
            file: "test.ts".to_string(),
            line: 1,
            column: 1,
            scope: "global".to_string(),
            block: "code".to_string(),
            source_code: None,
            span: Span::new(0, 4),
        };
        let output = OutputFormatter::format_type_errors(&[error], OutputFormat::Json);
        assert!(output.contains("TS100"));
    }

    #[test]
    #[serial]
    fn test_console_output_contains_ansi_colors() {
        // Clear any previous color settings and force enable colors for testing
        colored::control::unset_override();
        colored::control::set_override(true);

        let errors = vec![TypeError {
            id: "TS2322".to_string(),
            message: "Type mismatch".to_string(),
            file: "test.ts".to_string(),
            line: 42,
            column: 10,
            scope: "myFunction".to_string(),
            block: String::new(),
            source_code: None,
            span: Span::new(0, 10),
        }];

        let output = OutputFormatter::format_type_errors(&errors, OutputFormat::Console);

        // Should contain ANSI escape codes
        assert!(output.contains("\x1b["), "Output should contain ANSI escape sequences");
        assert!(output.contains("TS2322"), "Output should contain error ID");
        assert!(output.contains("test.ts"), "Output should contain file name");
        assert!(output.contains("myFunction"), "Output should contain scope");

        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    #[serial]
    fn test_console_output_specific_ansi_codes() {
        // Clear any previous color settings and force enable colors for testing
        colored::control::unset_override();
        colored::control::set_override(true);

        let errors = vec![TypeError {
            id: "TS2322".to_string(),
            message: "Type mismatch".to_string(),
            file: "test.ts".to_string(),
            line: 42,
            column: 10,
            scope: "myFunction".to_string(),
            block: String::new(),
            source_code: None,
            span: Span::new(0, 10),
        }];

        let output = OutputFormatter::format_type_errors(&errors, OutputFormat::Console);

        // RED (31), BLUE (34), CYAN (36) should be present
        // Note: colored crate may use combined codes like [1;31m for bold+red
        assert!(output.contains("\x1b[31m") || output.contains("\x1b[91m") || output.contains("\x1b[1;31m"), "Output should contain red color code. Got: {}", output);
        assert!(output.contains("\x1b[34m") || output.contains("\x1b[94m"), "Output should contain blue color code");
        assert!(output.contains("\x1b[36m") || output.contains("\x1b[96m"), "Output should contain cyan color code");

        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    fn test_html_output_contains_css_classes() {
        let errors = vec![TypeError {
            id: "TS2322".to_string(),
            message: "Type mismatch".to_string(),
            file: "test.ts".to_string(),
            line: 42,
            column: 10,
            scope: "myFunction".to_string(),
            block: String::new(),
            source_code: None,
            span: Span::new(0, 10),
        }];

        let output = OutputFormatter::format_type_errors(&errors, OutputFormat::Html);

        assert!(output.contains("class=\"error-id\""), "HTML should contain error-id class");
        assert!(output.contains("class=\"file-path\""), "HTML should contain file-path class");
        assert!(output.contains("class=\"scope\""), "HTML should contain scope class");
        assert!(output.contains("class=\"error-message\""), "HTML should contain error-message class");
    }

    #[test]
    #[serial]
    fn test_format_symbols_console_colorization() {
        // Clear any previous color settings and force enable colors for testing
        colored::control::unset_override();
        colored::control::set_override(true);

        let symbol = SymbolInfo {
            name: "MyClass".to_string(),
            kind: SymbolKind::Class,
            file: "test.ts".to_string(),
            start_line: 1,
            end_line: 10,
            exported: true,
            parameters: None,
            properties: None,
            return_type: None,
            jsdoc: None,
        };

        let output = OutputFormatter::format_symbols(&[symbol], OutputFormat::Console);

        // Should contain ANSI codes
        assert!(output.contains("\x1b["), "Symbols output should contain ANSI codes");
        assert!(output.contains("MyClass"), "Symbols output should contain class name");

        // CYAN (36) for name (bold), MAGENTA (35) for kind, BLUE (34) for file
        // Note: colored crate may use combined codes like [1;36m for bold+cyan
        assert!(output.contains("\x1b[36m") || output.contains("\x1b[96m") || output.contains("\x1b[1;36m"), "Should contain cyan for symbol name. Got: {}", output);
        assert!(output.contains("\x1b[35m") || output.contains("\x1b[95m"), "Should contain magenta for kind");
        assert!(output.contains("\x1b[34m") || output.contains("\x1b[94m"), "Should contain blue for file");

        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    fn test_format_symbols_html_classes() {
        use crate::models::{PropertyInfo, ParameterInfo};

        let symbol = SymbolInfo {
            name: "MyClass".to_string(),
            kind: SymbolKind::Class,
            file: "test.ts".to_string(),
            start_line: 1,
            end_line: 10,
            exported: true,
            parameters: Some(vec![ParameterInfo {
                name: "param1".to_string(),
                type_annotation: Some("string".to_string()),
                description: None,
            }]),
            properties: Some(vec![PropertyInfo {
                name: "prop1".to_string(),
                type_annotation: Some("number".to_string()),
                description: None,
            }]),
            return_type: None,
            jsdoc: None,
        };
        let output = OutputFormatter::format_symbols(&[symbol], OutputFormat::Html);

        assert!(output.contains("class=\"keyword\""), "HTML should contain keyword class");
        assert!(output.contains("class=\"symbol-name\""), "HTML should contain symbol-name class");
        assert!(output.contains("class=\"file-path\""), "HTML should contain file-path class");
        assert!(output.contains("class=\"type-annotation\""), "HTML should contain type-annotation class");
    }

    #[test]
    #[serial]
    fn test_symbols_with_parameters() {
        use crate::models::ParameterInfo;

        let symbol = SymbolInfo {
            name: "calculate".to_string(),
            kind: SymbolKind::Function,
            file: "math.ts".to_string(),
            start_line: 5,
            end_line: 10,
            exported: true,
            parameters: Some(vec![
                ParameterInfo {
                    name: "a".to_string(),
                    type_annotation: Some("number".to_string()),
                    description: None,
                },
                ParameterInfo {
                    name: "b".to_string(),
                    type_annotation: Some("number".to_string()),
                    description: None,
                },
            ]),
            properties: None,
            return_type: None,
            jsdoc: None,
        };
        // Clear any previous color settings and force enable colors for testing
        colored::control::unset_override();
        colored::control::set_override(true);

        let output = OutputFormatter::format_symbols(&[symbol], OutputFormat::Console);

        assert!(output.contains("calculate"), "Should contain function name");
        assert!(output.contains("a"), "Should contain parameter info");
        // YELLOW (33) for parameters
        assert!(output.contains("\x1b[33m") || output.contains("\x1b[93m"), "Should contain yellow for parameters. Got: {}", output);

        // Reset color override
        colored::control::unset_override();
    }
}
