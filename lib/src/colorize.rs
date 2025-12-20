use crate::models::TypeError;
use oxc_span::Span;

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

pub struct ConsoleColorizer;

impl ConsoleColorizer {
    pub fn colorize_code_block(code: &str, _language: &str) -> String {
        // Basic syntax highlighting for TypeScript/JavaScript
        // This is a naive implementation; a real lexer would be better for full syntax highlighting
        let mut colored = String::new();
        
        // Split by lines to handle comments properly if needed, but for now just process words
        let tokens = code.split_inclusive(|c: char| c.is_whitespace() || "{}()[],.;:".contains(c));
        
        for token in tokens {
            if token.starts_with("//") {
                colored.push_str(GREEN);
                colored.push_str(token);
                colored.push_str(RESET);
                continue;
            }

            // Check if the token ends with a separator
            let trimmed = token.trim();
            let is_keyword = matches!(trimmed, "const" | "let" | "var" | "function" | "class" | "interface" | "type" | "enum" | "import" | "export" | "from" | "return" | "if" | "else" | "for" | "while");
            let is_type = matches!(trimmed, "string" | "number" | "boolean" | "any" | "void" | "null" | "undefined");
            
            if is_keyword {
                colored.push_str(BLUE);
                colored.push_str(token);
                colored.push_str(RESET);
            } else if is_type {
                colored.push_str(CYAN);
                colored.push_str(token);
                colored.push_str(RESET);
            } else {
                colored.push_str(token);
            }
        }
        
        colored
    }

    pub fn highlight_error(_error_span: &Span, source: &str) -> String {
        // In a real implementation, we'd use the span to underline the error
        // For now, return the source with the whole block red for visibility
        format!("{}{}{}", RED, source, RESET)
    }
}

pub struct HtmlColorizer;

impl HtmlColorizer {
    pub fn colorize_code_block(code: &str, _language: &str) -> String {
        // Wrap tokens in spans
        let mut html = String::new();
        let tokens = code.split_inclusive(|c: char| c.is_whitespace() || "{}()[],.;:".contains(c));

        for token in tokens {
            let trimmed = token.trim();
            let is_keyword = matches!(trimmed, "const" | "let" | "var" | "function" | "class" | "interface" | "type" | "enum" | "import" | "export" | "from" | "return" | "if" | "else" | "for" | "while");
             let is_type = matches!(trimmed, "string" | "number" | "boolean" | "any" | "void" | "null" | "undefined");

             let escaped = html_escape::encode_text(token);

             if is_keyword {
                 html.push_str(&format!("<span class=\"keyword\">{}</span>", escaped));
             } else if is_type {
                 html.push_str(&format!("<span class=\"type\">{}</span>", escaped));
             } else {
                 html.push_str(&escaped);
             }
        }
        html
    }

    pub fn highlight_error(error: &TypeError, source: &str) -> String {
        format!(
            "<div class=\"error-block\" data-error-id=\"{}\"><pre>{}</pre><div class=\"message\">{}</div></div>",
            error.id,
            html_escape::encode_text(source),
            html_escape::encode_text(&error.message)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_colorizer() {
        let code = "const x: number = 5;";
        let colored = ConsoleColorizer::colorize_code_block(code, "ts");
        assert!(colored.contains(BLUE)); // const
        assert!(colored.contains(CYAN)); // number
        assert!(colored.contains(RESET));
    }

    #[test]
    fn test_html_colorizer() {
        let code = "const x: number = 5;";
        let html = HtmlColorizer::colorize_code_block(code, "ts");
        assert!(html.contains("<span class=\"keyword\">const"));
        assert!(html.contains("<span class=\"type\">number"));
    }
}
