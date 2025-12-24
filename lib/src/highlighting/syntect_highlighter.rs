use crate::highlighting::ansi::AnsiBuilder;
use crate::highlighting::error::{HighlightError, Result};
use crate::highlighting::options::HighlightOptions;
use crate::highlighting::themes::get_theme_by_name;
use crate::output::OutputFormat;
use serde::Serialize;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, FontStyle, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// A segment of highlighted code with styling information.
///
/// This struct separates data from rendering logic, allowing the same
/// highlighted code to be rendered in multiple formats.
#[derive(Debug, Clone, Serialize)]
pub struct HighlightSegment {
    /// The text content of this segment.
    pub text: String,

    /// The style to apply to this segment.
    pub style: SegmentStyle,

    /// The line number (1-indexed).
    pub line: usize,

    /// The column number (1-indexed).
    pub column: usize,
}

/// Style information for a code segment.
#[derive(Debug, Clone, Serialize)]
pub struct SegmentStyle {
    /// Foreground color (RGB).
    pub foreground: Option<RgbColor>,

    /// Background color (RGB).
    pub background: Option<RgbColor>,

    /// Whether the text is bold.
    pub bold: bool,

    /// Whether the text is italic.
    pub italic: bool,

    /// Whether the text is underlined.
    pub underline: bool,
}

/// An RGB color value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Color> for RgbColor {
    fn from(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

/// Highlighted code with segments and metadata.
///
/// This is the primary output structure from the highlighting system.
/// It can be rendered to Console (ANSI), HTML, or JSON formats.
#[derive(Debug, Clone, Serialize)]
pub struct HighlightedCode {
    /// The individual segments of highlighted code.
    pub segments: Vec<HighlightSegment>,

    /// The number of lines in the code.
    pub line_count: usize,

    /// The language used for highlighting.
    pub language: String,

    /// The theme used for highlighting.
    pub theme: String,

    /// Number of spaces to indent each line.
    pub indent_spaces: usize,
}

impl HighlightedCode {
    /// Renders the highlighted code as ANSI escape sequences for console output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::syntect_highlighter::HighlightedCode;
    /// # let code = HighlightedCode {
    /// #     segments: vec![],
    /// #     line_count: 1,
    /// #     language: "typescript".to_string(),
    /// #     theme: "Solarized (light)".to_string(),
    /// # };
    /// let console_output = code.render_console();
    /// // Contains ANSI escape codes like \x1b[38;2;R;G;Bm
    /// ```
    pub fn render_console(&self) -> String {
        let mut output = String::new();
        let indent = " ".repeat(self.indent_spaces);
        let mut line_start = true;

        for segment in &self.segments {
            // Add indentation at the start of each new line
            if line_start && self.indent_spaces > 0 {
                output.push_str(&indent);
                line_start = false;
            }

            if let Some(fg) = segment.style.foreground {
                let mut builder = AnsiBuilder::new().fg_rgb(fg.r, fg.g, fg.b);

                if segment.style.bold {
                    builder = builder.bold();
                }
                if segment.style.italic {
                    builder = builder.italic();
                }
                if segment.style.underline {
                    builder = builder.underline();
                }

                output.push_str(&builder.build());
                output.push_str(&segment.text);
                output.push_str(AnsiBuilder::RESET);
            } else {
                output.push_str(&segment.text);
            }

            // Check if this segment ends with a newline
            if segment.text.ends_with('\n') {
                line_start = true;
            }
        }

        output
    }

    /// Renders the highlighted code as HTML with semantic markup.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::syntect_highlighter::HighlightedCode;
    /// # let code = HighlightedCode {
    /// #     segments: vec![],
    /// #     line_count: 1,
    /// #     language: "typescript".to_string(),
    /// #     theme: "Solarized (light)".to_string(),
    /// # };
    /// let html_output = code.render_html();
    /// // Contains <span> elements with inline styles
    /// ```
    pub fn render_html(&self) -> String {
        let indent = " ".repeat(self.indent_spaces);
        let mut output = String::from("<pre><code>");
        let mut line_start = true;

        for segment in &self.segments {
            let text = html_escape::encode_text(&segment.text);

            // Add indentation at line start
            if line_start && self.indent_spaces > 0 {
                output.push_str(&html_escape::encode_text(&indent));
                line_start = false;
            }

            if segment.style.foreground.is_some() || segment.style.bold || segment.style.italic {
                let mut style_parts = Vec::new();

                if let Some(fg) = segment.style.foreground {
                    style_parts.push(format!("color: rgb({}, {}, {})", fg.r, fg.g, fg.b));
                }

                if segment.style.bold {
                    style_parts.push("font-weight: bold".to_string());
                }

                if segment.style.italic {
                    style_parts.push("font-style: italic".to_string());
                }

                if segment.style.underline {
                    style_parts.push("text-decoration: underline".to_string());
                }

                output.push_str(&format!(
                    r#"<span style="{}">{}</span>"#,
                    style_parts.join("; "),
                    text
                ));
            } else {
                output.push_str(text.as_ref());
            }

            // Check if this segment ends with a newline
            if segment.text.ends_with('\n') {
                line_start = true;
            }
        }

        output.push_str("</code></pre>");
        output
    }
}

/// Highlights code using syntect with the given options.
///
/// This is the core highlighting function that uses syntect's parsing
/// and highlighting engine.
///
/// # Errors
///
/// Returns `HighlightError` if:
/// - The language is not supported
/// - The theme cannot be loaded
/// - The code block exceeds the maximum size (10,000 lines)
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::syntect_highlighter::highlight_code;
/// use ta_lib::highlighting::HighlightOptions;
///
/// let code = "function hello() { return 'world'; }";
/// let options = HighlightOptions::new("js");
/// let highlighted = highlight_code(code, options)?;
/// # Ok::<(), ta_lib::highlighting::error::HighlightError>(())
/// ```
pub fn highlight_code(code: &str, options: HighlightOptions) -> Result<HighlightedCode> {
    // Enforce maximum code block size
    let line_count = code.lines().count();
    if line_count > 10_000 {
        return Err(HighlightError::CodeBlockTooLarge {
            size: line_count,
            max: 10_000,
        });
    }

    // Load syntax set
    let syntax_set = SyntaxSet::load_defaults_newlines();

    // Find syntax for the language
    // Try extension first (e.g., "ts", "rs", "py"), then token (e.g., "TypeScript")
    let syntax = syntax_set
        .find_syntax_by_extension(&options.language)
        .or_else(|| syntax_set.find_syntax_by_token(&options.language))
        .ok_or_else(|| HighlightError::UnsupportedLanguage(options.language.clone()))?;

    // Load theme
    let theme_name = match options.output_format {
        OutputFormat::Console | OutputFormat::Json => {
            options.dark_theme.as_deref().unwrap_or("base16-ocean.dark")
        }
        OutputFormat::Html => {
            options.light_theme.as_deref().unwrap_or("Solarized (light)")
        }
    };

    let theme = get_theme_by_name(theme_name)?;

    // Highlight the code
    let mut highlighter = HighlightLines::new(syntax, &theme);
    let mut segments = Vec::new();

    for (line_idx, line) in LinesWithEndings::from(code).enumerate() {
        let line_num = line_idx + 1;

        let highlighted = highlighter
            .highlight_line(line, &syntax_set)
            .map_err(|e| HighlightError::SyntectError(e.to_string()))?;

        let mut column = 1;
        for (style, text) in highlighted {
            let segment = HighlightSegment {
                text: text.to_string(),
                style: convert_style(style),
                line: line_num,
                column,
            };

            column += text.chars().count();
            segments.push(segment);
        }
    }

    Ok(HighlightedCode {
        segments,
        line_count,
        language: options.language.clone(),
        theme: theme_name.to_string(),
        indent_spaces: options.indent_spaces,
    })
}

/// Converts a syntect `Style` to our `SegmentStyle`.
fn convert_style(style: Style) -> SegmentStyle {
    SegmentStyle {
        foreground: Some(style.foreground.into()),
        background: if style.background.a > 0 {
            Some(style.background.into())
        } else {
            None
        },
        bold: style.font_style.contains(FontStyle::BOLD),
        italic: style.font_style.contains(FontStyle::ITALIC),
        underline: style.font_style.contains(FontStyle::UNDERLINE),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::OutputFormat;
    use syntect::parsing::SyntaxSet;

    #[test]
    fn test_syntect_language_detection() {
        // Debug test to see what syntect actually supports
        let ss = SyntaxSet::load_defaults_newlines();

        println!("\nBy extension .ts: {:?}", ss.find_syntax_by_extension("ts").map(|s| &s.name));
        println!("By extension .tsx: {:?}", ss.find_syntax_by_extension("tsx").map(|s| &s.name));
        println!("By token 'typescript': {:?}", ss.find_syntax_by_token("typescript").map(|s| &s.name));
        println!("By token 'TypeScript': {:?}", ss.find_syntax_by_token("TypeScript").map(|s| &s.name));

        // List syntaxes containing "Script"
        println!("\nSyntaxes containing 'Script':");
        for syntax in ss.syntaxes() {
            if syntax.name.contains("Script") {
                println!("  {} - extensions: {:?}", syntax.name, syntax.file_extensions);
            }
        }
    }

    #[test]
    fn test_highlight_typescript_code() {
        let code = "const x: number = 42;";
        let options = HighlightOptions::new("js");

        let result = highlight_code(code, options);
        assert!(result.is_ok());

        let highlighted = result.unwrap();
        assert_eq!(highlighted.language, "js");
        assert_eq!(highlighted.line_count, 1);
        assert!(!highlighted.segments.is_empty());
    }

    #[test]
    fn test_highlight_unsupported_language() {
        let code = "some code";
        let options = HighlightOptions::new("cobol");

        let result = highlight_code(code, options);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HighlightError::UnsupportedLanguage(_)
        ));
    }

    #[test]
    fn test_highlight_code_too_large() {
        let code = "line\n".repeat(10_001);
        let options = HighlightOptions::new("typescript");

        let result = highlight_code(&code, options);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HighlightError::CodeBlockTooLarge { .. }
        ));
    }

    #[test]
    fn test_render_console_contains_ansi() {
        let code = "const x = 42;";
        let options = HighlightOptions::new("js");

        let highlighted = highlight_code(code, options).unwrap();
        let console_output = highlighted.render_console();

        // Should contain ANSI escape sequences
        assert!(console_output.contains("\x1b[38;2;"));
    }

    #[test]
    fn test_render_html_contains_spans() {
        let code = "const x = 42;";
        let options = HighlightOptions::new("js")
            .for_format(OutputFormat::Html);

        let highlighted = highlight_code(code, options).unwrap();
        let html_output = highlighted.render_html();

        assert!(html_output.contains("<pre><code>"));
        assert!(html_output.contains("</code></pre>"));
        assert!(html_output.contains("<span"));
    }

    #[test]
    fn test_rgb_color_from_syntect() {
        let color = Color { r: 255, g: 128, b: 64, a: 255 };
        let rgb: RgbColor = color.into();

        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 128);
        assert_eq!(rgb.b, 64);
    }

    #[test]
    fn test_segment_style_bold() {
        let style = Style {
            foreground: Color { r: 255, g: 0, b: 0, a: 255 },
            background: Color { r: 0, g: 0, b: 0, a: 0 },
            font_style: FontStyle::BOLD,
        };

        let segment_style = convert_style(style);
        assert!(segment_style.bold);
        assert!(!segment_style.italic);
        assert!(!segment_style.underline);
    }

    #[test]
    fn test_segment_style_italic() {
        let style = Style {
            foreground: Color { r: 0, g: 255, b: 0, a: 255 },
            background: Color { r: 0, g: 0, b: 0, a: 0 },
            font_style: FontStyle::ITALIC,
        };

        let segment_style = convert_style(style);
        assert!(!segment_style.bold);
        assert!(segment_style.italic);
        assert!(!segment_style.underline);
    }

    #[test]
    fn test_multiline_code() {
        let code = "function add(a: number, b: number) {\n  return a + b;\n}";
        let options = HighlightOptions::new("js");

        let highlighted = highlight_code(code, options).unwrap();
        assert_eq!(highlighted.line_count, 3);

        // Verify line numbers in segments
        let line_1_segments: Vec<_> = highlighted.segments.iter().filter(|s| s.line == 1).collect();
        let line_2_segments: Vec<_> = highlighted.segments.iter().filter(|s| s.line == 2).collect();
        let line_3_segments: Vec<_> = highlighted.segments.iter().filter(|s| s.line == 3).collect();

        assert!(!line_1_segments.is_empty());
        assert!(!line_2_segments.is_empty());
        assert!(!line_3_segments.is_empty());
    }

    #[test]
    fn test_html_escapes_special_characters() {
        let code = "<script>alert('xss')</script>";
        let options = HighlightOptions::new("js")
            .for_format(OutputFormat::Html);

        let highlighted = highlight_code(code, options).unwrap();
        let html_output = highlighted.render_html();

        // Should not contain raw < or > characters (they should be escaped)
        assert!(!html_output.contains("<script>"));
        assert!(html_output.contains("&lt;script&gt;") || html_output.contains("&lt;"));
    }

    #[test]
    fn test_different_languages() {
        let test_cases = vec![
            ("js", "const x = 42;"),
            ("rs", "let x: i32 = 42;"),
            ("py", "x = 42"),
        ];

        for (lang, code) in test_cases {
            let options = HighlightOptions::new(lang);
            let result = highlight_code(code, options);
            assert!(result.is_ok(), "Failed to highlight {}", lang);
        }
    }

    #[test]
    fn test_theme_selection() {
        let code = "const x = 42;";

        // Console should use dark theme
        let options_console = HighlightOptions::new("js")
            .for_format(OutputFormat::Console);
        let highlighted_console = highlight_code(code, options_console).unwrap();
        assert_eq!(highlighted_console.theme, "base16-ocean.dark");

        // HTML should use light theme
        let options_html = HighlightOptions::new("js")
            .for_format(OutputFormat::Html);
        let highlighted_html = highlight_code(code, options_html).unwrap();
        assert_eq!(highlighted_html.theme, "Solarized (light)");
    }

    #[test]
    fn test_custom_theme() {
        let code = "const x = 42;";
        // Use a theme we know exists from the debug test
        let options = HighlightOptions::new("js")
            .with_theme("Solarized (light)");

        let highlighted = highlight_code(code, options).unwrap();
        assert_eq!(highlighted.theme, "Solarized (light)");
    }

    #[test]
    fn test_empty_code() {
        let code = "";
        let options = HighlightOptions::new("js");

        let result = highlight_code(code, options);
        assert!(result.is_ok());

        let highlighted = result.unwrap();
        assert_eq!(highlighted.line_count, 0);
    }

    #[test]
    fn test_code_with_only_whitespace() {
        let code = "   \n  \n   ";
        let options = HighlightOptions::new("js");

        let result = highlight_code(code, options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_highlighted_code_is_serializable() {
        let code = "const x = 42;";
        let options = HighlightOptions::new("js");

        let highlighted = highlight_code(code, options).unwrap();

        // Should be able to serialize to JSON
        let json = serde_json::to_string(&highlighted);
        assert!(json.is_ok());
    }
}
