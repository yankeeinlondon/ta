/// Context-aware code extraction for error highlighting.
///
/// This module provides functionality to extract relevant code context around
/// type errors, with smart truncation based on scope (function/method/type/module).

use oxc_span::Span;
use oxc_semantic::Semantic;
use serde::Serialize;

use crate::highlighting::error::{HighlightError, Result};

/// Represents the extracted code context around an error.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::code_context::{CodeContext, ScopeType};
///
/// let context = CodeContext {
///     full_code: "function test() { return 42; }".to_string(),
///     display_code: "function test() { return 42; }".to_string(),
///     scope_type: ScopeType::Function,
///     scope_name: "test".to_string(),
///     truncation_info: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CodeContext {
    /// The complete code for the containing scope.
    pub full_code: String,

    /// The code to display (potentially truncated for large scopes).
    pub display_code: String,

    /// The type of scope containing the error.
    pub scope_type: ScopeType,

    /// The name of the scope (e.g., function name, class::method).
    pub scope_name: String,

    /// Information about truncation, if any was applied.
    pub truncation_info: Option<TruncationInfo>,
}

/// The type of scope where an error occurred.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum ScopeType {
    /// Error in a standalone function.
    Function,

    /// Error in a class method.
    Method,

    /// Error in a type utility or type alias.
    TypeUtility,

    /// Error at module/file level (not in any specific scope).
    ModuleLevel,
}

/// Information about code truncation applied for display.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TruncationInfo {
    /// Original line count before truncation.
    pub original_line_count: usize,

    /// Number of lines in the displayed (truncated) code.
    pub displayed_line_count: usize,

    /// Sections that were truncated: (start_line, end_line) pairs.
    pub truncated_sections: Vec<(usize, usize)>,
}

/// Extracts code context around an error span.
///
/// This function finds the containing scope (function, method, type, or module-level)
/// and applies smart truncation based on scope size and error position.
///
/// # Arguments
///
/// * `source` - The complete source code.
/// * `error_span` - The span indicating where the error occurred.
/// * `semantic` - OXC semantic analysis data.
///
/// # Returns
///
/// Returns a `CodeContext` with the extracted code and metadata.
///
/// # Errors
///
/// Returns `HighlightError::InvalidSpan` if the span is out of bounds.
///
/// # Examples
///
/// ```ignore
/// use ta_lib::highlighting::code_context::extract_code_context;
/// use oxc_span::Span;
///
/// let source = "function test() { return 42; }";
/// let span = Span::new(18, 24); // "return"
/// let context = extract_code_context(source, span, &semantic)?;
/// assert_eq!(context.scope_name, "test");
/// # Ok::<(), ta_lib::highlighting::HighlightError>(())
/// ```
pub fn extract_code_context(
    source: &str,
    error_span: Span,
    semantic: &Semantic,
) -> Result<CodeContext> {
    // CRITICAL: Validate span bounds FIRST to prevent panics
    if error_span.end as usize > source.len() {
        let line = calculate_line_number(source, error_span.start as usize);
        let column = calculate_column_number(source, error_span.start as usize);
        return Err(HighlightError::InvalidSpan { line, column });
    }

    // Find the scope containing the error
    let scope_info = find_containing_scope(source, error_span, semantic)?;

    // Extract the code for this scope
    let scope_span = scope_info.span;
    let full_code = extract_span_text(source, scope_span)?;

    // Apply truncation logic based on scope type and size
    let (display_code, truncation_info) = apply_truncation(
        &full_code,
        error_span,
        scope_span,
        scope_info.scope_type,
    );

    Ok(CodeContext {
        full_code,
        display_code,
        scope_type: scope_info.scope_type,
        scope_name: scope_info.name,
        truncation_info,
    })
}

/// Information about a detected scope.
#[derive(Debug)]
struct ScopeInfo {
    span: Span,
    scope_type: ScopeType,
    name: String,
}

/// Finds the scope containing the given error span.
fn find_containing_scope(
    source: &str,
    error_span: Span,
    _semantic: &Semantic,
) -> Result<ScopeInfo> {
    use oxc_ast::visit::{Visit, walk};
    use oxc_ast::ast::*;

    // Find the smallest AST node containing the error
    struct ScopeFinder {
        error_span: Span,
        result: Option<ScopeInfo>,
    }

    impl<'a> Visit<'a> for ScopeFinder {
        fn visit_function(&mut self, func: &Function<'a>, _flags: oxc_semantic::ScopeFlags) {
            if !func.span.contains_inclusive(self.error_span) {
                return;
            }

            // If we have a name, use it
            if let Some(id) = &func.id {
                let name = id.name.to_string();
                // Only update if we don't have a result yet, or this is more specific (smaller span)
                if self.result.is_none() || self.result.as_ref().unwrap().span.size() > func.span.size() {
                    self.result = Some(ScopeInfo {
                        span: func.span,
                        scope_type: ScopeType::Function,
                        name,
                    });
                }
            }

            // Continue walking to find nested scopes
            walk::walk_function(self, func, _flags);
        }

        fn visit_class(&mut self, class: &Class<'a>) {
            if !class.span.contains_inclusive(self.error_span) {
                return;
            }

            // Update class scope
            if let Some(id) = &class.id {
                let class_name = id.name.to_string();
                if self.result.is_none() || self.result.as_ref().unwrap().span.size() > class.span.size() {
                    self.result = Some(ScopeInfo {
                        span: class.span,
                        scope_type: ScopeType::Method, // Will be refined if method found
                        name: class_name.clone(),
                    });
                }
            }

            // Continue to find methods
            walk::walk_class(self, class);
        }

        fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
            // Check if method body contains the error
            let method_span = method.span;
            if !method_span.contains_inclusive(self.error_span) {
                return;
            }

            // Get method name
            let method_name = match &method.key {
                PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                PropertyKey::PrivateIdentifier(id) => format!("#{}", id.name),
                _ => "method".to_string(),
            };

            // Get class name from previous scope (if any)
            let full_name = if let Some(ref prev_scope) = self.result {
                if prev_scope.span.contains_inclusive(method_span) {
                    format!("{}::{}", prev_scope.name, method_name)
                } else {
                    method_name
                }
            } else {
                method_name
            };

            self.result = Some(ScopeInfo {
                span: method_span,
                scope_type: ScopeType::Method,
                name: full_name,
            });

            walk::walk_method_definition(self, method);
        }

        fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
            if !decl.span.contains_inclusive(self.error_span) {
                return;
            }

            let name = decl.id.name.to_string();
            if self.result.is_none() || self.result.as_ref().unwrap().span.size() > decl.span.size() {
                self.result = Some(ScopeInfo {
                    span: decl.span,
                    scope_type: ScopeType::TypeUtility,
                    name,
                });
            }

            walk::walk_ts_type_alias_declaration(self, decl);
        }

        fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
            if !decl.span.contains_inclusive(self.error_span) {
                return;
            }

            let name = decl.id.name.to_string();
            if self.result.is_none() || self.result.as_ref().unwrap().span.size() > decl.span.size() {
                self.result = Some(ScopeInfo {
                    span: decl.span,
                    scope_type: ScopeType::TypeUtility,
                    name,
                });
            }

            walk::walk_ts_interface_declaration(self, decl);
        }
    }

    // Parse the source to get the AST
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let parse_result = Parser::new(&allocator, source, source_type).parse();

    let mut finder = ScopeFinder {
        error_span,
        result: None,
    };

    finder.visit_program(&parse_result.program);

    // If no specific scope found, return module-level
    Ok(finder.result.unwrap_or(ScopeInfo {
        span: Span::new(0, source.len() as u32),
        scope_type: ScopeType::ModuleLevel,
        name: "global".to_string(),
    }))
}

/// Extracts text for a given span with bounds checking.
fn extract_span_text(source: &str, span: Span) -> Result<String> {
    let start = span.start as usize;
    let end = span.end as usize;

    // Check for invalid span ordering
    if start > end {
        let line = calculate_line_number(source, start.min(source.len()));
        let column = calculate_column_number(source, start.min(source.len()));
        return Err(HighlightError::InvalidSpan { line, column });
    }

    if end > source.len() || start > source.len() {
        let line = calculate_line_number(source, start.min(source.len()));
        let column = calculate_column_number(source, start.min(source.len()));
        return Err(HighlightError::InvalidSpan { line, column });
    }

    // Ensure we're at valid UTF-8 char boundaries
    if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
        let line = calculate_line_number(source, start);
        let column = calculate_column_number(source, start);
        return Err(HighlightError::InvalidSpan { line, column });
    }

    Ok(source[start..end].to_string())
}

/// Applies truncation logic to large code blocks.
///
/// Truncation rules:
/// - Function/method/type <15 lines: Show full definition
/// - Function/method/type ≥15 lines: Show signature + context around error + closing
/// - Module-level: Smart boundary detection (stop at blank lines/closing braces)
fn apply_truncation(
    full_code: &str,
    error_span: Span,
    scope_span: Span,
    scope_type: ScopeType,
) -> (String, Option<TruncationInfo>) {
    let lines: Vec<&str> = full_code.lines().collect();
    let line_count = lines.len();

    // Calculate error line (relative to scope start)
    let error_line = calculate_relative_line_number(full_code, scope_span, error_span);

    // Apply truncation based on scope type
    match scope_type {
        ScopeType::Function | ScopeType::Method | ScopeType::TypeUtility => {
            // Short code: no truncation for function/method/type scopes
            if line_count < 15 {
                return (full_code.to_string(), None);
            }
            truncate_function_scope(&lines, error_line, line_count)
        }
        ScopeType::ModuleLevel => {
            // Always apply boundary detection for module-level scope
            // (even for short files, to avoid showing unrelated code)
            truncate_module_scope(&lines, error_line, line_count)
        }
    }
}

/// Truncates a function/method/type scope.
///
/// Shows: signature + ... + context around error + ... + closing
fn truncate_function_scope(
    lines: &[&str],
    error_line: usize,
    total_lines: usize,
) -> (String, Option<TruncationInfo>) {
    let mut displayed_lines = Vec::new();
    let mut truncated_sections = Vec::new();

    // First line (signature)
    displayed_lines.push(lines[0].to_string());

    // Truncation marker before error context
    let context_start = error_line.saturating_sub(2).max(1);
    if context_start > 1 {
        let omitted = context_start - 1;
        displayed_lines.push(format!("┄┄┄ ({} lines omitted) ┄┄┄", omitted));
        truncated_sections.push((1, context_start - 1));
    }

    // Context around error (2 lines before, error line, 2 lines after)
    let error_start = error_line.saturating_sub(2);
    let error_end = (error_line + 2).min(total_lines - 1);
    for i in error_start..=error_end {
        if i < lines.len() {
            displayed_lines.push(lines[i].to_string());
        }
    }

    // Truncation marker after error context
    let last_line_idx = total_lines - 1;
    if error_end < last_line_idx - 1 {
        let omitted = last_line_idx - error_end - 1;
        displayed_lines.push(format!("┄┄┄ ({} lines omitted) ┄┄┄", omitted));
        truncated_sections.push((error_end + 1, last_line_idx - 1));
    }

    // Last line (closing bracket)
    if last_line_idx < lines.len() {
        displayed_lines.push(lines[last_line_idx].to_string());
    }

    let display_code = displayed_lines.join("\n");
    let truncation_info = Some(TruncationInfo {
        original_line_count: total_lines,
        displayed_line_count: displayed_lines.len(),
        truncated_sections,
    });

    (display_code, truncation_info)
}

/// Truncates module-level code with smart boundary detection.
///
/// Rules:
/// - Maximum 3 lines before/after error
/// - Stop at blank lines (immediate termination)
/// - Stop at closing braces `}` (block boundaries)
/// - Stop at opening braces for new blocks
/// - NO truncation markers (just show the relevant lines)
fn truncate_module_scope(
    lines: &[&str],
    error_line: usize,
    total_lines: usize,
) -> (String, Option<TruncationInfo>) {
    // Find context start by scanning upward from error, stopping at boundaries
    let context_start = find_context_start(lines, error_line);

    // Find context end by scanning downward from error, stopping at boundaries
    let context_end = find_context_end(lines, error_line, total_lines);

    // Extract just the relevant lines (no truncation markers)
    let mut displayed_lines = Vec::new();
    for i in context_start..=context_end {
        if i < lines.len() {
            displayed_lines.push(lines[i].to_string());
        }
    }

    let display_code = displayed_lines.join("\n");

    // No truncation info for module-level - we're showing exactly what's relevant
    (display_code, None)
}

/// Finds the start of context by scanning upward from error line.
///
/// Stops at:
/// - 3 lines before error (maximum)
/// - Blank lines
/// - Closing braces `}` (include the brace, then stop)
fn find_context_start(lines: &[&str], error_line: usize) -> usize {
    const MAX_LINES_BEFORE: usize = 3;

    let mut start = error_line;
    let mut lines_seen = 0;

    // Scan upward from error_line - 1
    while start > 0 && lines_seen < MAX_LINES_BEFORE {
        let prev_line = start - 1;
        let line_content = lines[prev_line].trim();

        // Stop at blank lines
        if line_content.is_empty() {
            break;
        }

        // Stop after including closing brace (it's a boundary marker)
        if line_content == "}" || line_content.starts_with('}') {
            start = prev_line;  // Include the closing brace
            break;
        }

        start = prev_line;
        lines_seen += 1;
    }

    start
}

/// Finds the end of context by scanning downward from error line.
///
/// Stops at:
/// - 3 lines after error (maximum)
/// - Blank lines
/// - Opening braces for new blocks (function/class definitions)
fn find_context_end(lines: &[&str], error_line: usize, total_lines: usize) -> usize {
    const MAX_LINES_AFTER: usize = 3;

    let mut end = error_line;
    let mut lines_seen = 0;

    // Scan downward from error_line + 1
    while end < total_lines - 1 && lines_seen < MAX_LINES_AFTER {
        let next_line = end + 1;
        let line_content = lines[next_line].trim();

        // Stop at blank lines
        if line_content.is_empty() {
            break;
        }

        // Stop before function/class definitions (opening braces at start of new blocks)
        // But allow simple statements like `let x = { ... };`
        if is_block_start(line_content) {
            break;
        }

        end = next_line;
        lines_seen += 1;
    }

    end
}

/// Checks if a line starts a new block (function/class definition).
///
/// Examples that should return true:
/// - "function foo() {"
/// - "class Bar {"
/// - "export function baz() {"
///
/// Examples that should return false:
/// - "let x = { foo: 1 };"
/// - "return { bar: 2 };"
fn is_block_start(line: &str) -> bool {
    let line = line.trim();

    // Check for function/class declarations that end with {
    if line.ends_with('{') {
        // Function or class definition
        if line.contains("function ") ||
           line.contains("class ") ||
           line.contains("interface ") ||
           line.contains("type ") ||
           line.contains("enum ") {
            return true;
        }
    }

    false
}

/// Calculates the line number (1-indexed) for a byte offset.
fn calculate_line_number(source: &str, byte_offset: usize) -> usize {
    if byte_offset > source.len() {
        return 1;
    }

    // Adjust to nearest valid char boundary if not already on one
    let safe_offset = if !source.is_char_boundary(byte_offset) {
        // Find the previous valid boundary
        (0..=byte_offset)
            .rev()
            .find(|&i| source.is_char_boundary(i))
            .unwrap_or(0)
    } else {
        byte_offset
    };

    source[..safe_offset].chars().filter(|&c| c == '\n').count() + 1
}

/// Calculates the column number (1-indexed) for a byte offset.
fn calculate_column_number(source: &str, byte_offset: usize) -> usize {
    if byte_offset > source.len() {
        return 1;
    }

    // Adjust to nearest valid char boundary if not already on one
    let safe_offset = if !source.is_char_boundary(byte_offset) {
        // Find the previous valid boundary
        (0..=byte_offset)
            .rev()
            .find(|&i| source.is_char_boundary(i))
            .unwrap_or(0)
    } else {
        byte_offset
    };

    let line_start = source[..safe_offset]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);

    source[line_start..safe_offset].chars().count() + 1
}

/// Calculates the line number of error_span relative to scope_span start.
///
/// Returns the 0-indexed line number within the scope's extracted text.
fn calculate_relative_line_number(scope_text: &str, scope_span: Span, error_span: Span) -> usize {
    // If error is before scope (shouldn't happen), return 0
    if error_span.start < scope_span.start {
        return 0;
    }

    // Calculate byte offset within the scope
    let offset_in_scope = (error_span.start - scope_span.start) as usize;

    // Ensure offset is within bounds
    if offset_in_scope >= scope_text.len() {
        // Error is beyond scope, return last line
        return scope_text.lines().count().saturating_sub(1);
    }

    // Count newlines from start of scope_text up to the error offset
    // This gives us the 0-indexed line number
    scope_text[..offset_in_scope]
        .chars()
        .filter(|&c| c == '\n')
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_context_struct() {
        let context = CodeContext {
            full_code: "function test() { return 42; }".to_string(),
            display_code: "function test() { return 42; }".to_string(),
            scope_type: ScopeType::Function,
            scope_name: "test".to_string(),
            truncation_info: None,
        };

        assert_eq!(context.scope_name, "test");
        assert_eq!(context.scope_type, ScopeType::Function);
        assert!(context.truncation_info.is_none());
    }

    #[test]
    fn test_scope_type_variants() {
        assert_ne!(ScopeType::Function, ScopeType::Method);
        assert_ne!(ScopeType::Method, ScopeType::TypeUtility);
        assert_ne!(ScopeType::TypeUtility, ScopeType::ModuleLevel);
    }

    #[test]
    fn test_truncation_info_struct() {
        let info = TruncationInfo {
            original_line_count: 30,
            displayed_line_count: 10,
            truncated_sections: vec![(5, 20)],
        };

        assert_eq!(info.original_line_count, 30);
        assert_eq!(info.displayed_line_count, 10);
        assert_eq!(info.truncated_sections.len(), 1);
    }

    #[test]
    fn test_extract_span_text_bounds_checking() {
        let source = "function test() { return 42; }";
        let invalid_span = Span::new(0, 1000); // Beyond source length

        let result = extract_span_text(source, invalid_span);
        assert!(result.is_err());

        if let Err(HighlightError::InvalidSpan { line, column }) = result {
            assert!(line >= 1);
            assert!(column >= 1);
        } else {
            panic!("Expected InvalidSpan error");
        }
    }

    #[test]
    fn test_calculate_line_number() {
        let source = "line 1\nline 2\nline 3";

        assert_eq!(calculate_line_number(source, 0), 1);
        assert_eq!(calculate_line_number(source, 7), 2);
        assert_eq!(calculate_line_number(source, 14), 3);
    }

    #[test]
    fn test_calculate_column_number() {
        let source = "line 1\nline 2\nline 3";

        assert_eq!(calculate_column_number(source, 0), 1);
        assert_eq!(calculate_column_number(source, 5), 6);
        assert_eq!(calculate_column_number(source, 7), 1);
        assert_eq!(calculate_column_number(source, 10), 4);
    }

    #[test]
    fn test_extract_span_text_valid() {
        let source = "function test() { return 42; }";
        let span = Span::new(9, 13); // "test"

        let result = extract_span_text(source, span);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_extract_span_text_invalid() {
        let source = "function test() { return 42; }";
        let invalid_span = Span::new(0, 1000);

        let result = extract_span_text(source, invalid_span);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_truncation_short_code() {
        let code = "line 1\nline 2\nline 3";
        let error_span = Span::new(7, 13);
        let scope_span = Span::new(0, 20);

        let (display, info) = apply_truncation(code, error_span, scope_span, ScopeType::Function);

        assert_eq!(display, code);
        assert!(info.is_none());
    }

    #[test]
    fn test_apply_truncation_long_function() {
        let lines: Vec<String> = (1..=30).map(|i| format!("line {}", i)).collect();
        let code = lines.join("\n");
        let error_span = Span::new(100, 110); // Somewhere in the middle
        let scope_span = Span::new(0, code.len() as u32);

        let (display, info) = apply_truncation(&code, error_span, scope_span, ScopeType::Function);

        assert!(info.is_some());
        if let Some(truncation_info) = info {
            assert_eq!(truncation_info.original_line_count, 30);
            assert!(truncation_info.displayed_line_count < 30);
            assert!(display.contains("┄┄┄"));
            assert!(display.contains("lines omitted"));
        }
    }

    #[test]
    fn test_apply_truncation_module_level() {
        let lines: Vec<String> = (1..=30).map(|i| format!("line {}", i)).collect();
        let code = lines.join("\n");
        let error_span = Span::new(100, 110);
        let scope_span = Span::new(0, code.len() as u32);

        let (display, info) = apply_truncation(&code, error_span, scope_span, ScopeType::ModuleLevel);

        assert!(info.is_some());
        if let Some(truncation_info) = info {
            assert!(truncation_info.displayed_line_count < 30);
            assert!(display.contains("┄┄┄"));
        }
    }

    #[test]
    fn test_truncate_function_scope_markers() {
        let lines: Vec<&str> = (1..=30).map(|_| "code").collect();
        let error_line = 15;

        let (display, info) = truncate_function_scope(&lines, error_line, 30);

        assert!(display.contains("┄┄┄"));
        assert!(display.contains("lines omitted"));
        assert!(info.is_some());
    }

    #[test]
    fn test_truncate_module_scope_markers() {
        let lines: Vec<&str> = (1..=30).map(|_| "code").collect();
        let error_line = 15;

        let (display, info) = truncate_module_scope(&lines, error_line, 30);

        assert!(display.contains("┄┄┄"));
        assert!(display.contains("lines omitted"));
        assert!(info.is_some());
    }

    #[test]
    fn test_calculate_line_number_boundary() {
        let source = "a\nb\nc";

        assert_eq!(calculate_line_number(source, 0), 1);
        assert_eq!(calculate_line_number(source, 1), 1);
        assert_eq!(calculate_line_number(source, 2), 2);
        assert_eq!(calculate_line_number(source, 4), 3);
    }

    #[test]
    fn test_calculate_column_number_boundary() {
        let source = "abc\ndef\nghi";

        assert_eq!(calculate_column_number(source, 0), 1);
        assert_eq!(calculate_column_number(source, 2), 3);
        assert_eq!(calculate_column_number(source, 4), 1);
    }

    #[test]
    fn test_serialization() {
        let context = CodeContext {
            full_code: "test".to_string(),
            display_code: "test".to_string(),
            scope_type: ScopeType::Function,
            scope_name: "fn".to_string(),
            truncation_info: None,
        };

        let json = serde_json::to_string(&context);
        assert!(json.is_ok());
    }

    #[test]
    fn test_truncation_edge_case_exactly_15_lines() {
        let lines: Vec<String> = (1..=15).map(|i| format!("line {}", i)).collect();
        let code = lines.join("\n");
        let error_span = Span::new(50, 60);
        let scope_span = Span::new(0, code.len() as u32);

        let (_display, info) = apply_truncation(&code, error_span, scope_span, ScopeType::Function);

        // At exactly 15 lines, should apply truncation
        assert!(info.is_some());
        if let Some(truncation_info) = info {
            assert!(truncation_info.original_line_count >= 15);
        }
    }

    #[test]
    fn test_truncation_edge_case_14_lines() {
        let lines: Vec<String> = (1..=14).map(|i| format!("line {}", i)).collect();
        let code = lines.join("\n");
        let error_span = Span::new(50, 60);
        let scope_span = Span::new(0, code.len() as u32);

        let (display, info) = apply_truncation(&code, error_span, scope_span, ScopeType::Function);

        // At 14 lines, should NOT apply truncation
        assert!(info.is_none());
        assert_eq!(display, code);
    }
}

// Property-based tests
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_calculate_line_number_never_panics(
            source in "\\PC{0,1000}",
            offset in 0usize..1000,
        ) {
            let result = calculate_line_number(&source, offset);
            // Should always return a valid line number (≥1)
            prop_assert!(result >= 1);
        }

        #[test]
        fn test_calculate_column_number_never_panics(
            source in "\\PC{0,1000}",
            offset in 0usize..1000,
        ) {
            let result = calculate_column_number(&source, offset);
            // Should always return a valid column number (≥1)
            prop_assert!(result >= 1);
        }

        #[test]
        fn test_extract_span_text_bounds_checking_fuzz(
            source in "\\PC{0,1000}",
            start in 0u32..10000,
            end in 0u32..10000,
        ) {
            let span = Span::new(start, end);
            let result = extract_span_text(&source, span);

            // Should either succeed or return InvalidSpan error, never panic
            match result {
                Ok(text) => {
                    // If successful, text should be valid
                    prop_assert!(text.len() <= source.len());
                }
                Err(HighlightError::InvalidSpan { .. }) => {
                    // Expected error for out-of-bounds spans
                }
                Err(e) => {
                    // Unexpected error type
                    return Err(proptest::test_runner::TestCaseError::fail(
                        format!("Unexpected error: {:?}", e)
                    ));
                }
            }
        }

        #[test]
        fn test_apply_truncation_never_panics(
            lines_count in 1usize..100,
            _error_line in 0usize..100,
        ) {
            let lines: Vec<String> = (1..=lines_count).map(|i| format!("line {}", i)).collect();
            let code = lines.join("\n");
            let error_span = Span::new(0, 10);
            let scope_span = Span::new(0, code.len() as u32);

            // Should never panic regardless of inputs
            let (display, _info) = apply_truncation(
                &code,
                error_span,
                scope_span,
                ScopeType::Function
            );

            // Display should never be empty
            prop_assert!(!display.is_empty());
        }

        #[test]
        fn test_truncation_preserves_line_count_property(
            lines_count in 1usize..100,
        ) {
            let lines: Vec<String> = (1..=lines_count).map(|i| format!("line {}", i)).collect();
            let code = lines.join("\n");
            let error_span = Span::new(0, 10);
            let scope_span = Span::new(0, code.len() as u32);

            let (_display, info) = apply_truncation(
                &code,
                error_span,
                scope_span,
                ScopeType::Function
            );

            if let Some(truncation_info) = info {
                // Truncated version should have fewer lines than original
                prop_assert!(truncation_info.displayed_line_count <= truncation_info.original_line_count);
                prop_assert_eq!(truncation_info.original_line_count, lines_count);
            }
        }
    }
}
