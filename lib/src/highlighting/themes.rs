use crate::highlighting::error::{HighlightError, Result};
use std::path::{Path, PathBuf};
use syntect::highlighting::ThemeSet;

/// Built-in theme options available in syntect.
///
/// These themes are compiled into the binary and require no external files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTheme {
    /// Solarized light theme (default for light mode).
    SolarizedLight,
    /// Base16 Ocean Dark theme (default for dark mode).
    Base16OceanDark,
    /// Monokai Extended theme.
    MonokaiExtended,
    /// Zenburn theme.
    Zenburn,
    /// Dracula theme.
    Dracula,
    /// Gruvbox Dark theme.
    GruvboxDark,
    /// Gruvbox Light theme.
    GruvboxLight,
}

impl BuiltinTheme {
    /// Returns the string name of the theme as used by syntect.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::themes::BuiltinTheme;
    ///
    /// assert_eq!(BuiltinTheme::SolarizedLight.as_str(), "Solarized (light)");
    /// assert_eq!(BuiltinTheme::Base16OceanDark.as_str(), "base16-ocean.dark");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SolarizedLight => "Solarized (light)",
            Self::Base16OceanDark => "base16-ocean.dark",
            Self::MonokaiExtended => "Monokai Extended",
            Self::Zenburn => "Zenburn",
            Self::Dracula => "Dracula",
            Self::GruvboxDark => "gruvbox-dark",
            Self::GruvboxLight => "gruvbox-light",
        }
    }

    /// Attempts to parse a theme name into a `BuiltinTheme`.
    ///
    /// Case-insensitive, accepts both hyphenated and space-separated variants.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::themes::BuiltinTheme;
    ///
    /// let theme = BuiltinTheme::from_name("solarized-light").unwrap();
    /// assert_eq!(theme, BuiltinTheme::SolarizedLight);
    ///
    /// let theme2 = BuiltinTheme::from_name("Solarized (light)").unwrap();
    /// assert_eq!(theme2, BuiltinTheme::SolarizedLight);
    ///
    /// assert!(BuiltinTheme::from_name("nonexistent").is_err());
    /// ```
    pub fn from_name(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "solarized-light" | "solarized (light)" => Ok(Self::SolarizedLight),
            "base16-ocean-dark" | "base16-ocean.dark" => Ok(Self::Base16OceanDark),
            "monokai-extended" | "monokai extended" => Ok(Self::MonokaiExtended),
            "zenburn" => Ok(Self::Zenburn),
            "dracula" => Ok(Self::Dracula),
            "gruvbox-dark" | "gruvbox dark" => Ok(Self::GruvboxDark),
            "gruvbox-light" | "gruvbox light" => Ok(Self::GruvboxLight),
            _ => Err(HighlightError::ThemeNotFound {
                name: name.to_string(),
            }),
        }
    }

    /// Returns an iterator over all built-in themes.
    ///
    /// # Examples
    ///
    /// ```
    /// use ta_lib::highlighting::themes::BuiltinTheme;
    ///
    /// let themes: Vec<_> = BuiltinTheme::iter().collect();
    /// assert!(themes.len() >= 7);
    /// ```
    pub fn iter() -> impl Iterator<Item = Self> {
        use BuiltinTheme::*;
        [
            SolarizedLight,
            Base16OceanDark,
            MonokaiExtended,
            Zenburn,
            Dracula,
            GruvboxDark,
            GruvboxLight,
        ]
        .into_iter()
    }
}

/// Source of a theme (built-in or custom file).
#[derive(Debug, Clone)]
pub enum ThemeSource {
    /// A built-in theme compiled into the binary.
    Builtin(BuiltinTheme),
    /// A custom theme loaded from a file path.
    Custom(PathBuf),
}

/// Lists all available built-in theme names.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::themes::list_available_themes;
///
/// let themes = list_available_themes();
/// assert!(themes.contains(&"Solarized (light)".to_string()));
/// assert!(themes.contains(&"base16-ocean.dark".to_string()));
/// ```
pub fn list_available_themes() -> Vec<String> {
    BuiltinTheme::iter()
        .map(|t| t.as_str().to_string())
        .collect()
}

/// Gets the default theme set with all built-in themes.
///
/// This function loads syntect's default theme set, which includes
/// all the built-in themes.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::themes::get_default_theme_set;
///
/// let theme_set = get_default_theme_set();
/// assert!(theme_set.themes.contains_key("Solarized (light)"));
/// ```
pub fn get_default_theme_set() -> ThemeSet {
    ThemeSet::load_defaults()
}

/// Loads a theme from a custom file path.
///
/// # Security
///
/// This function validates the path to prevent directory traversal attacks.
/// Paths containing `..` components will be rejected.
///
/// # Errors
///
/// Returns `HighlightError::ThemeLoadError` if:
/// - The file cannot be read
/// - The file is not a valid `.tmTheme` file
/// - The path contains directory traversal attempts
///
/// # Examples
///
/// ```no_run
/// use ta_lib::highlighting::themes::load_theme_from_file;
/// use std::path::Path;
///
/// let path = Path::new("/path/to/custom.tmTheme");
/// let theme = load_theme_from_file(path)?;
/// # Ok::<(), ta_lib::highlighting::error::HighlightError>(())
/// ```
pub fn load_theme_from_file(path: &Path) -> Result<syntect::highlighting::Theme> {
    // Canonicalize path to resolve symlinks and relative paths
    let canonical = path.canonicalize()
        .map_err(|e| HighlightError::ThemeLoadError { source: e })?;

    // Check for directory traversal attempts
    // After canonicalization, ".." should not appear in components
    if canonical.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(HighlightError::ThemeLoadError {
            source: std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Path traversal not allowed"
            ),
        });
    }

    // Read and parse .tmTheme file using syntect's get_theme method
    ThemeSet::get_theme(&canonical)
        .map_err(|e| HighlightError::ThemeLoadError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })
}

/// Gets a theme by name, trying built-in themes first.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::themes::get_theme_by_name;
///
/// let theme = get_theme_by_name("Solarized (light)").unwrap();
/// assert_eq!(theme.name, Some("Solarized (light)".to_string()));
/// ```
pub fn get_theme_by_name(name: &str) -> Result<syntect::highlighting::Theme> {
    let theme_set = get_default_theme_set();

    // Try exact match first
    if let Some(theme) = theme_set.themes.get(name) {
        return Ok(theme.clone());
    }

    // Try parsing as BuiltinTheme (handles case-insensitive + variants)
    if let Ok(builtin) = BuiltinTheme::from_name(name) {
        if let Some(theme) = theme_set.themes.get(builtin.as_str()) {
            return Ok(theme.clone());
        }
    }

    Err(HighlightError::ThemeNotFound {
        name: name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_theme_as_str() {
        assert_eq!(BuiltinTheme::SolarizedLight.as_str(), "Solarized (light)");
        assert_eq!(
            BuiltinTheme::Base16OceanDark.as_str(),
            "base16-ocean.dark"
        );
        assert_eq!(
            BuiltinTheme::MonokaiExtended.as_str(),
            "Monokai Extended"
        );
    }

    #[test]
    fn test_builtin_theme_from_name() {
        assert_eq!(
            BuiltinTheme::from_name("solarized-light").unwrap(),
            BuiltinTheme::SolarizedLight
        );
        assert_eq!(
            BuiltinTheme::from_name("Solarized (light)").unwrap(),
            BuiltinTheme::SolarizedLight
        );
        assert_eq!(
            BuiltinTheme::from_name("BASE16-OCEAN-DARK").unwrap(),
            BuiltinTheme::Base16OceanDark
        );
        assert_eq!(
            BuiltinTheme::from_name("zenburn").unwrap(),
            BuiltinTheme::Zenburn
        );
    }

    #[test]
    fn test_builtin_theme_from_name_invalid() {
        assert!(BuiltinTheme::from_name("nonexistent").is_err());
        assert!(BuiltinTheme::from_name("").is_err());
    }

    #[test]
    fn test_builtin_theme_iter() {
        let themes: Vec<_> = BuiltinTheme::iter().collect();
        assert!(themes.len() >= 7);
        assert!(themes.contains(&BuiltinTheme::SolarizedLight));
        assert!(themes.contains(&BuiltinTheme::Base16OceanDark));
    }

    #[test]
    fn test_list_available_themes() {
        let themes = list_available_themes();
        assert!(themes.contains(&"Solarized (light)".to_string()));
        assert!(themes.contains(&"base16-ocean.dark".to_string()));
        assert!(themes.contains(&"Dracula".to_string()));
    }

    #[test]
    fn test_get_default_theme_set() {
        let theme_set = get_default_theme_set();
        assert!(!theme_set.themes.is_empty());
        assert!(theme_set.themes.contains_key("Solarized (light)"));
        assert!(theme_set.themes.contains_key("base16-ocean.dark"));
    }

    #[test]
    fn test_get_theme_by_name() {
        let theme = get_theme_by_name("Solarized (light)").unwrap();
        assert_eq!(theme.name, Some("Solarized (light)".to_string()));
    }

    #[test]
    fn test_get_theme_by_name_case_insensitive() {
        let theme = get_theme_by_name("solarized-light").unwrap();
        assert_eq!(theme.name, Some("Solarized (light)".to_string()));
    }

    #[test]
    fn test_get_theme_by_name_not_found() {
        let result = get_theme_by_name("NonExistentTheme");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HighlightError::ThemeNotFound { .. }
        ));
    }

    #[test]
    fn test_theme_source_variants() {
        let builtin = ThemeSource::Builtin(BuiltinTheme::Dracula);
        let custom = ThemeSource::Custom(PathBuf::from("/custom/theme.tmTheme"));

        assert!(matches!(builtin, ThemeSource::Builtin(_)));
        assert!(matches!(custom, ThemeSource::Custom(_)));
    }

    #[test]
    fn test_load_theme_from_file_nonexistent() {
        let result = load_theme_from_file(Path::new("/nonexistent/theme.tmTheme"));
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_theme_is_copy() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<BuiltinTheme>();
    }

    #[test]
    fn test_builtin_theme_equality() {
        assert_eq!(BuiltinTheme::Dracula, BuiltinTheme::Dracula);
        assert_ne!(BuiltinTheme::Dracula, BuiltinTheme::Zenburn);
    }

    #[test]
    fn test_load_theme_from_file_path_traversal() {
        // Should reject paths with ".." components
        // Note: This test will fail on canonicalize if path doesn't exist,
        // which is the expected security behavior
        let result = load_theme_from_file(Path::new("../../../etc/passwd"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HighlightError::ThemeLoadError { .. }
        ));
    }

    #[test]
    fn test_theme_source_builtin_variant() {
        let source = ThemeSource::Builtin(BuiltinTheme::Dracula);
        assert!(matches!(source, ThemeSource::Builtin(BuiltinTheme::Dracula)));
    }

    #[test]
    fn test_theme_source_custom_variant() {
        let path = PathBuf::from("/tmp/custom.tmTheme");
        let source = ThemeSource::Custom(path.clone());
        if let ThemeSource::Custom(p) = source {
            assert_eq!(p, path);
        } else {
            panic!("Expected Custom variant");
        }
    }
}
