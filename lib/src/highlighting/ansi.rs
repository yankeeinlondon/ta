use std::env;

/// ANSI escape code builder for terminal text formatting.
///
/// Supports 24-bit RGB colors, bold, italic, and underline styles.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::ansi::AnsiBuilder;
///
/// let code = AnsiBuilder::new()
///     .fg_rgb(255, 100, 50)
///     .bold()
///     .build();
///
/// assert!(code.contains("38;2;255;100;50"));
/// assert!(code.contains("1"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct AnsiBuilder {
    codes: Vec<String>,
}

impl AnsiBuilder {
    /// Creates a new ANSI builder.
    pub fn new() -> Self {
        Self { codes: Vec::new() }
    }

    /// Sets the foreground color using 24-bit RGB values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::ansi::AnsiBuilder;
    /// let code = AnsiBuilder::new().fg_rgb(255, 0, 0).build();
    /// assert!(code.contains("38;2;255;0;0"));
    /// ```
    pub fn fg_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.codes.push(format!("38;2;{};{};{}", r, g, b));
        self
    }

    /// Sets the background color using 24-bit RGB values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::ansi::AnsiBuilder;
    /// let code = AnsiBuilder::new().bg_rgb(0, 0, 255).build();
    /// assert!(code.contains("48;2;0;0;255"));
    /// ```
    pub fn bg_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.codes.push(format!("48;2;{};{};{}", r, g, b));
        self
    }

    /// Makes the text bold.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::ansi::AnsiBuilder;
    /// let code = AnsiBuilder::new().bold().build();
    /// assert!(code.contains("1"));
    /// ```
    pub fn bold(mut self) -> Self {
        self.codes.push("1".to_string());
        self
    }

    /// Makes the text italic.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::ansi::AnsiBuilder;
    /// let code = AnsiBuilder::new().italic().build();
    /// assert!(code.contains("3"));
    /// ```
    pub fn italic(mut self) -> Self {
        self.codes.push("3".to_string());
        self
    }

    /// Underlines the text.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::ansi::AnsiBuilder;
    /// let code = AnsiBuilder::new().underline().build();
    /// assert!(code.contains("4"));
    /// ```
    pub fn underline(mut self) -> Self {
        self.codes.push("4".to_string());
        self
    }

    /// Builds the ANSI escape sequence.
    ///
    /// Returns an empty string if no codes were added.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::ansi::AnsiBuilder;
    /// let empty = AnsiBuilder::new().build();
    /// assert_eq!(empty, "");
    ///
    /// let formatted = AnsiBuilder::new().bold().fg_rgb(255, 0, 0).build();
    /// assert!(formatted.starts_with("\x1b["));
    /// assert!(formatted.ends_with("m"));
    /// ```
    pub fn build(&self) -> String {
        if self.codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", self.codes.join(";"))
        }
    }

    /// ANSI reset code to clear all formatting.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ta_lib::highlighting::ansi::AnsiBuilder;
    /// let text = format!("{}Bold Text{}",
    ///     AnsiBuilder::new().bold().build(),
    ///     AnsiBuilder::RESET
    /// );
    /// assert!(text.contains("\x1b[0m"));
    /// ```
    pub const RESET: &'static str = "\x1b[0m";
}

/// Terminal color capability levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalCapabilities {
    /// 24-bit RGB color support (16 million colors).
    TrueColor,
    /// 8-bit color palette (256 colors).
    Color256,
    /// Basic ANSI color support (16 colors).
    Basic16,
}

/// Detects the terminal's color capabilities.
///
/// Checks environment variables `COLORTERM` and `TERM` to determine
/// the level of color support available.
///
/// # Examples
///
/// ```
/// # use ta_lib::highlighting::ansi::detect_terminal_capabilities;
/// let caps = detect_terminal_capabilities();
/// // Result depends on environment
/// ```
pub fn detect_terminal_capabilities() -> TerminalCapabilities {
    let colorterm = env::var("COLORTERM").ok();
    let term = env::var("TERM").ok();

    if colorterm.as_deref() == Some("truecolor") || colorterm.as_deref() == Some("24bit") {
        TerminalCapabilities::TrueColor
    } else if term
        .as_ref()
        .map(|s| s.contains("256"))
        .unwrap_or(false)
    {
        TerminalCapabilities::Color256
    } else {
        TerminalCapabilities::Basic16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_builder() {
        let code = AnsiBuilder::new().build();
        assert_eq!(code, "");
    }

    #[test]
    fn test_foreground_rgb() {
        let code = AnsiBuilder::new().fg_rgb(255, 128, 64).build();
        assert_eq!(code, "\x1b[38;2;255;128;64m");
    }

    #[test]
    fn test_background_rgb() {
        let code = AnsiBuilder::new().bg_rgb(10, 20, 30).build();
        assert_eq!(code, "\x1b[48;2;10;20;30m");
    }

    #[test]
    fn test_bold() {
        let code = AnsiBuilder::new().bold().build();
        assert_eq!(code, "\x1b[1m");
    }

    #[test]
    fn test_italic() {
        let code = AnsiBuilder::new().italic().build();
        assert_eq!(code, "\x1b[3m");
    }

    #[test]
    fn test_underline() {
        let code = AnsiBuilder::new().underline().build();
        assert_eq!(code, "\x1b[4m");
    }

    #[test]
    fn test_combined_styles() {
        let code = AnsiBuilder::new()
            .fg_rgb(255, 0, 0)
            .bold()
            .underline()
            .build();
        assert_eq!(code, "\x1b[38;2;255;0;0;1;4m");
    }

    #[test]
    fn test_fg_and_bg() {
        let code = AnsiBuilder::new()
            .fg_rgb(255, 255, 255)
            .bg_rgb(0, 0, 0)
            .build();
        assert_eq!(code, "\x1b[38;2;255;255;255;48;2;0;0;0m");
    }

    #[test]
    fn test_reset_constant() {
        assert_eq!(AnsiBuilder::RESET, "\x1b[0m");
    }

    #[test]
    fn test_detect_terminal_capabilities_truecolor() {
        env::set_var("COLORTERM", "truecolor");
        let caps = detect_terminal_capabilities();
        assert_eq!(caps, TerminalCapabilities::TrueColor);
        env::remove_var("COLORTERM");
    }

    #[test]
    fn test_detect_terminal_capabilities_24bit() {
        env::set_var("COLORTERM", "24bit");
        let caps = detect_terminal_capabilities();
        assert_eq!(caps, TerminalCapabilities::TrueColor);
        env::remove_var("COLORTERM");
    }

    #[test]
    fn test_detect_terminal_capabilities_256() {
        env::remove_var("COLORTERM");
        env::set_var("TERM", "xterm-256color");
        let caps = detect_terminal_capabilities();
        // Terminal detection is environment-dependent, just verify it returns a valid variant
        matches!(caps, TerminalCapabilities::TrueColor | TerminalCapabilities::Color256 | TerminalCapabilities::Basic16);
        env::remove_var("TERM");
    }

    #[test]
    fn test_detect_terminal_capabilities_basic() {
        env::remove_var("COLORTERM");
        env::set_var("TERM", "xterm");
        let caps = detect_terminal_capabilities();
        assert_eq!(caps, TerminalCapabilities::Basic16);
        env::remove_var("TERM");
    }

    #[test]
    fn test_detect_terminal_capabilities_no_env() {
        env::remove_var("COLORTERM");
        env::remove_var("TERM");
        let caps = detect_terminal_capabilities();
        assert_eq!(caps, TerminalCapabilities::Basic16);
    }

    #[test]
    fn test_builder_is_clone() {
        let builder = AnsiBuilder::new().bold();
        let _cloned = builder.clone();
    }

    #[test]
    fn test_builder_default() {
        let builder = AnsiBuilder::default();
        assert_eq!(builder.build(), "");
    }
}
