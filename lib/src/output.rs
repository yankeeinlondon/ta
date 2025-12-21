use crate::models::{SymbolInfo, TypeError};
use crate::colorize::{ConsoleColorizer, HtmlColorizer, RED, RESET, BOLD};
use serde::Serialize;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Console,
    Html,
    Json,
}

pub struct OutputFormatter;

impl OutputFormatter {
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
            output.push_str(&format!(
                "{}{} :{} - {}error {}{} {}{}\n",
                BOLD,
                error.file,
                error.line,
                RED,
                error.id,
                RESET,
                error.message,
                RESET
            ));
            
            output.push_str(&format!(
                "    {}\n",
                ConsoleColorizer::highlight_error(&error.span, &error.block)
            ));
            output.push('\n');
        }
        output
    }

    fn format_type_errors_html(errors: &[TypeError]) -> String {
        let mut output = String::from("<div class=\"type-errors\">");
        for error in errors {
            output.push_str(&HtmlColorizer::highlight_error(error, &error.block));
        }
        output.push_str("</div>");
        output
    }

    fn format_symbols_console(symbols: &[SymbolInfo]) -> String {
        let mut output = String::new();
        for symbol in symbols {
            output.push_str(&format!(
                "{:?} {} in {}:{}-{}\n",
                symbol.kind,
                symbol.name,
                symbol.file,
                symbol.start_line,
                symbol.end_line
            ));
            if let Some(props) = &symbol.properties {
                for prop in props {
                    output.push_str(&format!("  - {}\n", prop.name));
                }
            }
        }
        output
    }

    fn format_symbols_html(symbols: &[SymbolInfo]) -> String {
        let mut output = String::from("<ul class=\"symbols\">");
        for symbol in symbols {
            output.push_str(&format!(
                "<li data-kind=\"{:?}\">{} ({})</li>",
                symbol.kind,
                html_escape::encode_text(&symbol.name),
                html_escape::encode_text(&symbol.file)
            ));
        }
        output.push_str("</ul>");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_span::Span;
    use crate::models::SymbolKind;

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
            span: Span::new(0, 4),
        };
        let output = OutputFormatter::format_type_errors(&[error], OutputFormat::Json);
        assert!(output.contains("TS100"));
    }

    #[test]
    fn test_format_symbols_console() {
        let symbol = SymbolInfo {
            name: "MyClass".to_string(),
            kind: SymbolKind::Class,
            file: "test.ts".to_string(),
            start_line: 1,
            end_line: 10,
            exported: true,
            parameters: None,
            properties: None,
        };
        let output = OutputFormatter::format_symbols(&[symbol], OutputFormat::Console);
        assert!(output.contains("Class MyClass in test.ts"));
    }
}
