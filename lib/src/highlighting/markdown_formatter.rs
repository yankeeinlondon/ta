//! Markdown parsing and formatting with embedded code highlighting.
//!
//! This module implements markdown parsing using pulldown-cmark and integrates
//! with the code highlighting system to provide syntax-highlighted code blocks
//! with language indicators, titles, and visual separators.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag, TagEnd};

use crate::highlighting::{highlight_code, HighlightOptions, MarkdownOptions, Result};
use crate::output::OutputFormat;

/// Formatted markdown with embedded code highlighting.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FormattedMarkdown {
    /// The formatted output ready for display.
    pub output: String,

    /// The output format used.
    pub format: OutputFormat,

    /// Number of code blocks found and highlighted.
    pub code_block_count: usize,
}

impl FormattedMarkdown {
    /// Creates a new formatted markdown result.
    pub fn new(output: String, format: OutputFormat, code_block_count: usize) -> Self {
        Self {
            output,
            format,
            code_block_count,
        }
    }

    /// Returns the formatted output as a string.
    pub fn as_str(&self) -> &str {
        &self.output
    }
}

/// Formats markdown text with embedded code highlighting.
///
/// This function parses markdown using pulldown-cmark and highlights code blocks
/// using the syntect highlighter. It supports:
/// - Language indicators extracted from code fence info strings
/// - Optional code block titles from info strings
/// - Visual separators (box-drawing characters for console output)
/// - Fallback to plain text for unknown languages
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::{format_markdown, MarkdownOptions};
/// use ta_lib::output::OutputFormat;
///
/// # fn main() -> Result<(), ta_lib::highlighting::HighlightError> {
/// let markdown = "# Hello\n\nWorld";
///
/// let options = MarkdownOptions::default();
/// let result = format_markdown(markdown, options)?;
/// assert!(result.output.contains("Hello"));
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if code highlighting fails for a code block.
pub fn format_markdown(text: &str, options: MarkdownOptions) -> Result<FormattedMarkdown> {
    let parser = Parser::new(text);
    let mut formatter = MarkdownFormatter::new(options);

    for event in parser {
        formatter.process_event(event)?;
    }

    let code_block_count = formatter.code_block_count;
    let format = formatter.options.output_format;
    let output = formatter.finalize();

    Ok(FormattedMarkdown::new(output, format, code_block_count))
}

/// Internal markdown formatter state machine.
struct MarkdownFormatter {
    /// Formatting options.
    options: MarkdownOptions,

    /// Output buffer.
    output: String,

    /// Current state.
    state: FormatterState,

    /// Number of code blocks processed.
    code_block_count: usize,

    /// Current code block being accumulated.
    current_code: String,

    /// Current code block language.
    current_language: Option<String>,

    /// Current code block title.
    current_title: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormatterState {
    /// Processing prose/regular markdown.
    Prose,

    /// Inside a code block.
    CodeBlock,

    /// Inside a heading.
    Heading(usize),

    /// Inside a paragraph.
    Paragraph,

    /// Inside a list.
    List,
}

impl MarkdownFormatter {
    fn new(options: MarkdownOptions) -> Self {
        Self {
            options,
            output: String::new(),
            state: FormatterState::Prose,
            code_block_count: 0,
            current_code: String::new(),
            current_language: None,
            current_title: None,
        }
    }

    fn process_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag)?,
            Event::End(tag_end) => self.handle_end_tag(tag_end)?,
            Event::Text(text) => self.handle_text(text)?,
            Event::Code(code) => self.handle_inline_code(code),
            Event::SoftBreak => self.output.push(' '),
            Event::HardBreak => self.output.push('\n'),
            Event::Rule => self.handle_rule(),
            _ => {}
        }
        Ok(())
    }

    fn handle_start_tag(&mut self, tag: Tag) -> Result<()> {
        match tag {
            Tag::CodeBlock(kind) => {
                self.state = FormatterState::CodeBlock;
                let (lang, title) = parse_code_block_info(kind);
                self.current_language = lang;
                self.current_title = title;
                self.current_code.clear();
            }
            Tag::Heading { level, .. } => {
                self.state = FormatterState::Heading(level as usize);
                self.output.push_str("\n\n");
            }
            Tag::Paragraph => {
                self.state = FormatterState::Paragraph;
                self.output.push_str("\n\n");
            }
            Tag::List(_) => {
                self.state = FormatterState::List;
                self.output.push('\n');
            }
            Tag::Item => {
                self.output.push_str("\n  • ");
            }
            Tag::Emphasis => {
                if self.options.output_format == OutputFormat::Html {
                    self.output.push_str("<em>");
                }
            }
            Tag::Strong => {
                if self.options.output_format == OutputFormat::Html {
                    self.output.push_str("<strong>");
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_end_tag(&mut self, tag_end: TagEnd) -> Result<()> {
        match tag_end {
            TagEnd::CodeBlock => {
                self.flush_code_block()?;
                self.state = FormatterState::Prose;
            }
            TagEnd::Heading(_) => {
                self.output.push('\n');
                self.state = FormatterState::Prose;
            }
            TagEnd::Paragraph => {
                self.state = FormatterState::Prose;
            }
            TagEnd::List(_) => {
                self.output.push('\n');
                self.state = FormatterState::Prose;
            }
            TagEnd::Emphasis => {
                if self.options.output_format == OutputFormat::Html {
                    self.output.push_str("</em>");
                }
            }
            TagEnd::Strong => {
                if self.options.output_format == OutputFormat::Html {
                    self.output.push_str("</strong>");
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_text(&mut self, text: CowStr) -> Result<()> {
        match self.state {
            FormatterState::CodeBlock => {
                self.current_code.push_str(&text);
            }
            FormatterState::Heading(level) => {
                self.format_heading(&text, level);
            }
            _ => {
                self.output.push_str(&text);
            }
        }
        Ok(())
    }

    fn handle_inline_code(&mut self, code: CowStr) {
        match self.options.output_format {
            OutputFormat::Console => {
                self.output.push('`');
                self.output.push_str(&code);
                self.output.push('`');
            }
            OutputFormat::Html => {
                self.output.push_str("<code>");
                self.output
                    .push_str(html_escape::encode_text(&code).as_ref());
                self.output.push_str("</code>");
            }
            OutputFormat::Json => {
                self.output.push_str(&code);
            }
        }
    }

    fn handle_rule(&mut self) {
        match self.options.output_format {
            OutputFormat::Console => {
                self.output.push_str("\n\n───────────────────────────────────────\n\n");
            }
            OutputFormat::Html => {
                self.output.push_str("\n<hr>\n");
            }
            OutputFormat::Json => {
                self.output.push_str("\n---\n");
            }
        }
    }

    fn format_heading(&mut self, text: &str, level: usize) {
        match self.options.output_format {
            OutputFormat::Console => {
                let prefix = "#".repeat(level);
                self.output.push_str(&prefix);
                self.output.push(' ');
                self.output.push_str(text);
            }
            OutputFormat::Html => {
                self.output.push_str(&format!("<h{}>", level));
                self.output
                    .push_str(html_escape::encode_text(text).as_ref());
                self.output.push_str(&format!("</h{}>", level));
            }
            OutputFormat::Json => {
                self.output.push_str(text);
            }
        }
    }

    fn flush_code_block(&mut self) -> Result<()> {
        let code = self.current_code.clone();
        let language = self
            .current_language
            .as_deref()
            .unwrap_or("text")
            .to_string();
        let title = self.current_title.take();

        self.code_block_count += 1;

        match self.options.output_format {
            OutputFormat::Console => {
                self.output.push_str("\n\n");
                self.render_code_block_console(&code, &language, title.as_deref())?;
            }
            OutputFormat::Html => {
                self.output.push('\n');
                self.render_code_block_html(&code, &language, title.as_deref())?;
            }
            OutputFormat::Json => {
                self.output.push_str("\n```");
                self.output.push_str(&language);
                if let Some(t) = title {
                    self.output.push(' ');
                    self.output.push_str(&t);
                }
                self.output.push('\n');
                self.output.push_str(&code);
                self.output.push_str("\n```\n");
            }
        }

        self.current_code.clear();
        self.current_language = None;

        Ok(())
    }

    fn render_code_block_console(
        &mut self,
        code: &str,
        language: &str,
        title: Option<&str>,
    ) -> Result<()> {
        let highlight_opts = HighlightOptions::new(language)
            .with_line_numbers(self.options.show_line_numbers)
            .for_format(OutputFormat::Console);

        // Try to highlight, fall back to plain text on error
        let highlighted = match highlight_code(code, highlight_opts) {
            Ok(h) => h.render_console(),
            Err(_) => code.to_string(),
        };

        // Render visual separator with language and title
        let header = format_code_block_header_console(language, title);
        self.output.push_str(&header);
        self.output.push('\n');

        // Add the highlighted code
        self.output.push_str(&highlighted);

        // Bottom border
        self.output
            .push_str("└───────────────────────────────────────────────────────────┘\n");

        Ok(())
    }

    fn render_code_block_html(
        &mut self,
        code: &str,
        language: &str,
        title: Option<&str>,
    ) -> Result<()> {
        let highlight_opts = HighlightOptions::new(language)
            .with_line_numbers(self.options.show_line_numbers)
            .for_format(OutputFormat::Html);

        // Try to highlight, fall back to plain text on error
        let highlighted = match highlight_code(code, highlight_opts) {
            Ok(h) => h.render_html(),
            Err(_) => format!(
                "<pre><code>{}</code></pre>",
                html_escape::encode_text(code)
            ),
        };

        // Render code block with header
        self.output.push_str("<div class=\"code-block\">\n");

        if title.is_some() || !language.is_empty() {
            self.output
                .push_str("  <div class=\"code-block__header\">\n");

            if let Some(t) = title {
                self.output
                    .push_str(&format!("    <span class=\"code-block__title\">{}</span>\n", t));
            }

            if !language.is_empty() {
                self.output.push_str(&format!(
                    "    <span class=\"code-block__language\" data-lang=\"{}\">{}</span>\n",
                    language, language
                ));
            }

            self.output.push_str("  </div>\n");
        }

        self.output.push_str(&highlighted);
        self.output.push_str("</div>\n");

        Ok(())
    }

    fn finalize(self) -> String {
        self.output.trim().to_string()
    }
}

/// Parses code block info string to extract language and title.
///
/// Info string format: "language Title Text"
/// - First word: language identifier
/// - Remaining words: optional title
///
/// # Examples
///
/// ```
/// # use ta_lib::highlighting::markdown_formatter::parse_code_block_info;
/// # use pulldown_cmark::CodeBlockKind;
/// let kind = CodeBlockKind::Fenced("ts My Function".into());
/// let (lang, title) = parse_code_block_info(kind);
/// assert_eq!(lang, Some("ts".to_string()));
/// assert_eq!(title, Some("My Function".to_string()));
/// ```
pub fn parse_code_block_info(kind: CodeBlockKind) -> (Option<String>, Option<String>) {
    match kind {
        CodeBlockKind::Fenced(info) => {
            let info_str = info.trim();
            if info_str.is_empty() {
                return (None, None);
            }

            let parts: Vec<&str> = info_str.split_whitespace().collect();
            let language = parts.first().map(|s| s.to_string());
            let title = if parts.len() > 1 {
                Some(parts[1..].join(" "))
            } else {
                None
            };

            (language, title)
        }
        CodeBlockKind::Indented => (Some("text".to_string()), None),
    }
}

/// Formats a code block header for console output using box-drawing characters.
fn format_code_block_header_console(language: &str, title: Option<&str>) -> String {
    let lang_part = if !language.is_empty() {
        format!(" {} ", language)
    } else {
        String::new()
    };

    let title_part = if let Some(t) = title {
        format!(" ─ {} ", t)
    } else {
        String::new()
    };

    let header_content = format!("{}{}", lang_part, title_part);
    let padding_needed = 60_usize.saturating_sub(header_content.len() + 2);
    let padding = "─".repeat(padding_needed);

    format!("┌─{}{}─┐", header_content, padding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::OutputFormat;

    #[test]
    fn test_parse_code_block_info_with_language_and_title() {
        let kind = CodeBlockKind::Fenced("ts My TypeScript Function".into());
        let (lang, title) = parse_code_block_info(kind);
        assert_eq!(lang, Some("ts".to_string()));
        assert_eq!(title, Some("My TypeScript Function".to_string()));
    }

    #[test]
    fn test_parse_code_block_info_language_only() {
        let kind = CodeBlockKind::Fenced("javascript".into());
        let (lang, title) = parse_code_block_info(kind);
        assert_eq!(lang, Some("javascript".to_string()));
        assert_eq!(title, None);
    }

    #[test]
    fn test_parse_code_block_info_empty() {
        let kind = CodeBlockKind::Fenced("".into());
        let (lang, title) = parse_code_block_info(kind);
        assert_eq!(lang, None);
        assert_eq!(title, None);
    }

    #[test]
    fn test_parse_code_block_info_indented() {
        let kind = CodeBlockKind::Indented;
        let (lang, title) = parse_code_block_info(kind);
        assert_eq!(lang, Some("text".to_string()));
        assert_eq!(title, None);
    }

    #[test]
    fn test_format_markdown_basic() {
        let markdown = r#"# Hello World

This is a paragraph.
"#;
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options).unwrap();

        assert!(result.output.contains("Hello World"));
        assert!(result.output.contains("paragraph"));
        assert_eq!(result.code_block_count, 0);
    }

    #[test]
    fn test_format_markdown_with_code_block() {
        let markdown = r#"# Test

```js
console.log("hello");
```
"#;
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options).unwrap();

        assert!(result.output.contains("Test"));
        assert_eq!(result.code_block_count, 1);
    }

    #[test]
    fn test_format_markdown_code_block_with_title() {
        let markdown = r#"```ts Example Function
function test() {
    return 42;
}
```"#;
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options).unwrap();

        assert_eq!(result.code_block_count, 1);
        assert!(result.output.contains("ts"));
    }

    #[test]
    fn test_format_markdown_multiple_code_blocks() {
        let markdown = r#"# Code Examples

```js
const x = 1;
```

Some text.

```ts
const y = 2;
```
"#;
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options).unwrap();

        assert_eq!(result.code_block_count, 2);
    }

    #[test]
    fn test_format_markdown_unknown_language_fallback() {
        let markdown = r#"```unknownlang
some code
```"#;
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options).unwrap();

        // Should not error, fallback to plain text
        assert_eq!(result.code_block_count, 1);
        assert!(result.output.contains("some code"));
    }

    #[test]
    fn test_format_markdown_no_language() {
        let markdown = r#"```
plain text
```"#;
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options).unwrap();

        assert_eq!(result.code_block_count, 1);
        assert!(result.output.contains("plain text"));
    }

    #[test]
    fn test_format_markdown_inline_code() {
        let markdown = "This is `inline code` in text.";
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options).unwrap();

        assert!(result.output.contains("`inline code`"));
    }

    #[test]
    fn test_format_markdown_html_output() {
        let markdown = r#"```js Test
const x = 1;
```"#;
        let options = MarkdownOptions {
            output_format: OutputFormat::Html,
            ..Default::default()
        };
        let result = format_markdown(markdown, options).unwrap();

        assert!(result.output.contains("code-block"));
        assert!(result.output.contains("data-lang=\"js\""));
        assert!(result.output.contains("Test"));
    }

    #[test]
    fn test_format_markdown_nested_in_list() {
        let markdown = r#"- Item 1
- Item 2
  - Nested item"#;
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options).unwrap();

        assert!(result.output.contains("Item 1"));
        assert!(result.output.contains("Item 2"));
    }

    #[test]
    fn test_format_markdown_horizontal_rule() {
        let markdown = r#"Text before

---

Text after"#;
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options).unwrap();

        assert!(result.output.contains("Text before"));
        assert!(result.output.contains("Text after"));
    }

    #[test]
    fn test_format_markdown_with_line_numbers() {
        let markdown = r#"```ts
function test() {
    return 42;
}
```"#;
        let options = MarkdownOptions {
            show_line_numbers: true,
            ..Default::default()
        };
        let result = format_markdown(markdown, options).unwrap();

        assert_eq!(result.code_block_count, 1);
    }

    #[test]
    fn test_format_code_block_header_console() {
        let header = format_code_block_header_console("ts", Some("My Function"));
        assert!(header.starts_with("┌─"));
        assert!(header.ends_with("─┐"));
        assert!(header.contains("ts"));
        assert!(header.contains("My Function"));
    }

    #[test]
    fn test_format_code_block_header_console_no_title() {
        let header = format_code_block_header_console("js", None);
        assert!(header.starts_with("┌─"));
        assert!(header.ends_with("─┐"));
        assert!(header.contains("js"));
    }

    #[test]
    fn test_formatted_markdown_accessors() {
        let fm = FormattedMarkdown::new("output".to_string(), OutputFormat::Console, 3);
        assert_eq!(fm.as_str(), "output");
        assert_eq!(fm.code_block_count, 3);
        assert_eq!(fm.format, OutputFormat::Console);
    }

    #[test]
    fn test_malformed_markdown_graceful() {
        // Unclosed emphasis should be handled gracefully
        let markdown = "*unclosed emphasis";
        let options = MarkdownOptions::default();
        let result = format_markdown(markdown, options);

        assert!(result.is_ok());
    }
}
