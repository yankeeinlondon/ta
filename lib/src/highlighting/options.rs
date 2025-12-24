use crate::output::OutputFormat;

/// Options for highlighting code with syntax highlighting and error annotations.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::HighlightOptions;
/// use ta_lib::output::OutputFormat;
///
/// let options = HighlightOptions::new("typescript")
///     .with_line_numbers(true)
///     .for_format(OutputFormat::Console);
/// ```
#[derive(Debug, Clone)]
pub struct HighlightOptions {
    /// The programming language to use for syntax highlighting.
    pub language: String,

    /// Optional light theme name (defaults to "Solarized (light)").
    pub light_theme: Option<String>,

    /// Optional dark theme name (defaults to "base16-ocean.dark").
    pub dark_theme: Option<String>,

    /// Whether to show line numbers in the output.
    pub show_line_numbers: bool,

    /// Number of spaces to indent the entire code block.
    pub indent_spaces: usize,

    /// Error spans to annotate in the code (populated in Phase 2).
    /// For Phase 1, this is a placeholder Vec<()>.
    pub error_spans: Vec<()>, // TODO: Replace with Vec<ErrorAnnotation> in Phase 2

    /// The output format (Console, HTML, or JSON).
    pub output_format: OutputFormat,
}

impl Default for HighlightOptions {
    /// Creates default highlighting options.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::HighlightOptions;
    ///
    /// let options = HighlightOptions::default();
    /// assert_eq!(options.language, "js");
    /// assert!(!options.show_line_numbers);
    /// ```
    fn default() -> Self {
        Self {
            language: "js".to_string(), // JavaScript (TypeScript not in default syntect)
            light_theme: None, // Will use "Solarized (light)"
            dark_theme: None,  // Will use "base16-ocean.dark"
            show_line_numbers: false,
            indent_spaces: 0,  // No indentation by default
            error_spans: Vec::new(),
            output_format: OutputFormat::Console,
        }
    }
}

impl HighlightOptions {
    /// Creates new highlighting options for a specific language.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::HighlightOptions;
    ///
    /// let options = HighlightOptions::new("rust");
    /// assert_eq!(options.language, "rust");
    /// ```
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
            ..Default::default()
        }
    }

    /// Sets the theme for both light and dark modes.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::HighlightOptions;
    ///
    /// let options = HighlightOptions::new("typescript")
    ///     .with_theme("Monokai Extended");
    ///
    /// assert_eq!(options.light_theme, Some("Monokai Extended".to_string()));
    /// assert_eq!(options.dark_theme, Some("Monokai Extended".to_string()));
    /// ```
    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        let theme = theme.into();
        self.light_theme = Some(theme.clone());
        self.dark_theme = Some(theme);
        self
    }

    /// Sets whether to show line numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::HighlightOptions;
    ///
    /// let options = HighlightOptions::new("typescript")
    ///     .with_line_numbers(true);
    ///
    /// assert!(options.show_line_numbers);
    /// ```
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Sets the number of spaces to indent the entire code block.
    ///
    /// This is useful for visually nesting code blocks within error messages.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::HighlightOptions;
    ///
    /// let options = HighlightOptions::new("js")
    ///     .with_indent(2);  // Indent by 2 spaces
    ///
    /// assert_eq!(options.indent_spaces, 2);
    /// ```
    pub fn with_indent(mut self, spaces: usize) -> Self {
        self.indent_spaces = spaces;
        self
    }

    /// Sets the output format.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::HighlightOptions;
    /// use ta_lib::output::OutputFormat;
    ///
    /// let options = HighlightOptions::new("typescript")
    ///     .for_format(OutputFormat::Html);
    ///
    /// assert!(matches!(options.output_format, OutputFormat::Html));
    /// ```
    pub fn for_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }
}

/// Options for formatting markdown with embedded code highlighting.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::MarkdownOptions;
/// use ta_lib::output::OutputFormat;
///
/// let options = MarkdownOptions::default()
///     .with_line_numbers(true)
///     .for_format(OutputFormat::Html);
/// ```
#[derive(Debug, Clone)]
pub struct MarkdownOptions {
    /// Optional light theme for code blocks.
    pub code_light_theme: Option<String>,

    /// Optional dark theme for code blocks.
    pub code_dark_theme: Option<String>,

    /// Whether to show line numbers in code blocks.
    pub show_line_numbers: bool,

    /// The output format (Console, HTML, or JSON).
    pub output_format: OutputFormat,
}

impl Default for MarkdownOptions {
    /// Creates default markdown formatting options.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::MarkdownOptions;
    ///
    /// let options = MarkdownOptions::default();
    /// assert!(!options.show_line_numbers);
    /// ```
    fn default() -> Self {
        Self {
            code_light_theme: None,
            code_dark_theme: None,
            show_line_numbers: false,
            output_format: OutputFormat::Console,
        }
    }
}

impl MarkdownOptions {
    /// Creates new markdown options.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::MarkdownOptions;
    ///
    /// let options = MarkdownOptions::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the theme for code blocks in both light and dark modes.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::MarkdownOptions;
    ///
    /// let options = MarkdownOptions::new()
    ///     .with_code_theme("Dracula");
    ///
    /// assert_eq!(options.code_light_theme, Some("Dracula".to_string()));
    /// ```
    pub fn with_code_theme(mut self, theme: impl Into<String>) -> Self {
        let theme = theme.into();
        self.code_light_theme = Some(theme.clone());
        self.code_dark_theme = Some(theme);
        self
    }

    /// Sets whether to show line numbers in code blocks.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::MarkdownOptions;
    ///
    /// let options = MarkdownOptions::new()
    ///     .with_line_numbers(true);
    ///
    /// assert!(options.show_line_numbers);
    /// ```
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Sets the output format.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::MarkdownOptions;
    /// use ta_lib::output::OutputFormat;
    ///
    /// let options = MarkdownOptions::new()
    ///     .for_format(OutputFormat::Json);
    ///
    /// assert!(matches!(options.output_format, OutputFormat::Json));
    /// ```
    pub fn for_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_options_default() {
        let options = HighlightOptions::default();
        assert_eq!(options.language, "js");
        assert_eq!(options.light_theme, None);
        assert_eq!(options.dark_theme, None);
        assert!(!options.show_line_numbers);
        assert!(options.error_spans.is_empty());
    }

    #[test]
    fn test_highlight_options_new() {
        let options = HighlightOptions::new("rust");
        assert_eq!(options.language, "rust");
    }

    #[test]
    fn test_highlight_options_with_theme() {
        let options = HighlightOptions::new("typescript").with_theme("Monokai Extended");
        assert_eq!(
            options.light_theme,
            Some("Monokai Extended".to_string())
        );
        assert_eq!(options.dark_theme, Some("Monokai Extended".to_string()));
    }

    #[test]
    fn test_highlight_options_with_line_numbers() {
        let options = HighlightOptions::new("typescript").with_line_numbers(true);
        assert!(options.show_line_numbers);
    }

    #[test]
    fn test_highlight_options_for_format() {
        let options = HighlightOptions::new("typescript").for_format(OutputFormat::Html);
        assert!(matches!(options.output_format, OutputFormat::Html));
    }

    #[test]
    fn test_highlight_options_builder_chain() {
        let options = HighlightOptions::new("rust")
            .with_theme("Dracula")
            .with_line_numbers(true)
            .for_format(OutputFormat::Json);

        assert_eq!(options.language, "rust");
        assert_eq!(options.light_theme, Some("Dracula".to_string()));
        assert!(options.show_line_numbers);
        assert!(matches!(options.output_format, OutputFormat::Json));
    }

    #[test]
    fn test_markdown_options_default() {
        let options = MarkdownOptions::default();
        assert_eq!(options.code_light_theme, None);
        assert_eq!(options.code_dark_theme, None);
        assert!(!options.show_line_numbers);
    }

    #[test]
    fn test_markdown_options_new() {
        let options = MarkdownOptions::new();
        assert!(!options.show_line_numbers);
    }

    #[test]
    fn test_markdown_options_with_code_theme() {
        let options = MarkdownOptions::new().with_code_theme("Zenburn");
        assert_eq!(options.code_light_theme, Some("Zenburn".to_string()));
        assert_eq!(options.code_dark_theme, Some("Zenburn".to_string()));
    }

    #[test]
    fn test_markdown_options_with_line_numbers() {
        let options = MarkdownOptions::new().with_line_numbers(true);
        assert!(options.show_line_numbers);
    }

    #[test]
    fn test_markdown_options_for_format() {
        let options = MarkdownOptions::new().for_format(OutputFormat::Html);
        assert!(matches!(options.output_format, OutputFormat::Html));
    }

    #[test]
    fn test_markdown_options_builder_chain() {
        let options = MarkdownOptions::new()
            .with_code_theme("Solarized (light)")
            .with_line_numbers(true)
            .for_format(OutputFormat::Console);

        assert_eq!(
            options.code_light_theme,
            Some("Solarized (light)".to_string())
        );
        assert!(options.show_line_numbers);
        assert!(matches!(options.output_format, OutputFormat::Console));
    }

    #[test]
    fn test_options_are_clone() {
        let options1 = HighlightOptions::new("typescript");
        let _options2 = options1.clone();

        let md_options1 = MarkdownOptions::new();
        let _md_options2 = md_options1.clone();
    }
}
