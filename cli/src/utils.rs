//! Utility functions for CLI argument processing

/// Normalize a user-provided pattern by adding wildcards if not already glob-like
///
/// This function examines the pattern for glob syntax characters at the start
/// and end (`*`, `?`, `[`, `{` at start; `*`, `?`, `]`, `}` at end) and adds
/// wildcards as needed to make the pattern more flexible.
///
/// # Examples
///
/// ```rust
/// use ta::utils::normalize_glob_pattern;
///
/// // Plain strings get wildcards on both ends
/// assert_eq!(normalize_glob_pattern("foobar"), "*foobar*");
///
/// // Patterns with glob syntax are preserved
/// assert_eq!(normalize_glob_pattern("src/**/*.ts"), "src/**/*.ts");
///
/// // Partial wildcards get completed
/// assert_eq!(normalize_glob_pattern("*Class"), "*Class*");
/// ```
pub fn normalize_glob_pattern(pattern: &str) -> String {
    // If pattern contains path separators or **, it's already a full glob pattern - leave as-is
    if pattern.contains('/') || pattern.contains("**") {
        return pattern.to_string();
    }

    // If pattern starts with { or [, it's already a complete glob pattern - leave as-is
    if pattern.starts_with('{') || pattern.starts_with('[') {
        return pattern.to_string();
    }

    let has_glob_start = pattern.starts_with('*')
        || pattern.starts_with('?');

    let has_glob_end = pattern.ends_with('*')
        || pattern.ends_with('?');

    match (has_glob_start, has_glob_end) {
        (true, true) => pattern.to_string(),         // Already has wildcards both ends
        (true, false) => format!("{}*", pattern),    // Add wildcard at end
        (false, true) => format!("*{}", pattern),    // Add wildcard at start
        (false, false) => format!("*{}*", pattern),  // Add wildcards both ends
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_glob_pattern_no_wildcards() {
        // Pattern with no glob syntax should get wildcards added
        assert_eq!(normalize_glob_pattern("foobar"), "*foobar*");
        assert_eq!(normalize_glob_pattern("MyClass"), "*MyClass*");
        assert_eq!(normalize_glob_pattern("test"), "*test*");
    }

    #[test]
    fn test_normalize_glob_pattern_with_start_wildcard() {
        // Pattern starting with wildcard should only add end wildcard
        assert_eq!(normalize_glob_pattern("*Class"), "*Class*");
        assert_eq!(normalize_glob_pattern("?oo"), "?oo*");
    }

    #[test]
    fn test_normalize_glob_pattern_with_end_wildcard() {
        // Pattern ending with wildcard should only add start wildcard
        assert_eq!(normalize_glob_pattern("Class*"), "*Class*");
        assert_eq!(normalize_glob_pattern("foo?"), "*foo?");
    }

    #[test]
    fn test_normalize_glob_pattern_both_wildcards() {
        // Pattern with wildcards on both ends should remain unchanged
        assert_eq!(normalize_glob_pattern("*foobar*"), "*foobar*");
        assert_eq!(normalize_glob_pattern("**/test/*"), "**/test/*");
        assert_eq!(normalize_glob_pattern("src/**/*.ts"), "src/**/*.ts");
    }

    #[test]
    fn test_normalize_glob_pattern_with_braces() {
        // Pattern with braces should be preserved
        assert_eq!(normalize_glob_pattern("{test,spec}"), "{test,spec}");
        assert_eq!(normalize_glob_pattern("*.{ts,tsx}"), "*.{ts,tsx}*");
    }

    #[test]
    fn test_normalize_glob_pattern_with_brackets() {
        // Pattern with brackets should be preserved
        assert_eq!(normalize_glob_pattern("[abc]"), "[abc]");
        assert_eq!(normalize_glob_pattern("test[123]"), "*test[123]*");
    }

    #[test]
    fn test_normalize_glob_pattern_complex() {
        // Complex glob patterns should work correctly
        assert_eq!(
            normalize_glob_pattern("src/**/*.test.{ts,tsx}"),
            "src/**/*.test.{ts,tsx}"
        );
        assert_eq!(normalize_glob_pattern("**/foobar/**"), "**/foobar/**");
    }
}
