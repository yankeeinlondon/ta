# Code and Markdown Highlighting Implementation Plan

**Created:** 2025-12-23
**Status:** Reviewed - Ready for Implementation
**Last Updated:** 2025-12-23 (Post-Review)

## Executive Summary

This plan implements comprehensive code and markdown highlighting for the TypeScript Analyzer (TA) using `syntect` (TextMate grammars) and `pulldown-cmark` for markdown parsing. The implementation will add two core utility functions: `highlight_code()` for syntax highlighting and `format_markdown()` for markdown-aware highlighting with code blocks. Both functions will support three output formats (Console/HTML/JSON) with configurable themes, line numbers, error annotations, and interactive features for HTML output.

**Review Status:** All three sub-agent reviews (Rust Developer, Schema Architect, Feature Tester) returned "Approve with Changes". All mandatory changes have been incorporated into this updated plan.

## Requirements

### Functional Requirements

| ID | Requirement | Priority | Owner |
|----|-------------|----------|-------|
| FR-1 | Implement `highlight_code(code, options)` for TypeScript syntax highlighting | High | Rust Developer |
| FR-2 | Implement `format_markdown(text, options)` for markdown parsing with embedded code highlighting | High | Rust Developer |
| FR-3 | Support Console output with ANSI 24-bit color escape codes | High | Rust Developer |
| FR-4 | Support HTML output with CSS classes and semantic markup | High | Rust Developer |
| FR-5 | Support JSON output with structured data | Medium | Rust Developer |
| FR-6 | Light and dark theme support for both code and text | High | Schema Architect |
| FR-7 | Optional line numbers for code blocks | Medium | Rust Developer |
| FR-8 | Error highlighting with "red squigglies" and annotations | High | Rust Developer |
| FR-9 | HTML popover support for error descriptions | Medium | Rust Developer |
| FR-10 | Code block language indicators with optional titles | High | Rust Developer |
| FR-11 | Interactive clipboard copy for HTML code blocks | Low | Rust Developer |
| FR-12 | Integrate `highlight_code()` into `ta source` command | High | Rust Developer |
| FR-13 | Context-aware code snippet extraction (function/method scope) | High | Rust Developer |

### Non-Functional Requirements

| ID | Requirement | Target | Owner |
|----|-------------|--------|-------|
| NFR-1 | Syntax highlighting performance | <10ms per code block (<1000 lines) | Rust Developer |
| NFR-2 | Theme loading and caching | Load once, cache in memory | Rust Developer |
| NFR-3 | Memory efficiency | Use syntect's cached syntax sets | Rust Developer |
| NFR-4 | Color accuracy | Support full 24-bit RGB color space | Rust Developer |
| NFR-5 | HTML output security | Escape all user content to prevent XSS | Rust Developer |
| NFR-6 | Type safety | Strongly-typed theme and options structs | Schema Architect |
| NFR-7 | Error handling | Graceful fallback for unsupported languages | Rust Developer |
| NFR-8 | Test coverage | ≥85% for highlighting modules | Feature Tester (Rust) |

## Architecture Overview

The implementation will extend the existing `lib/src/` module structure with new highlighting capabilities:

```
lib/src/
├── highlighting/
│   ├── mod.rs              # Public API (highlight_code, format_markdown)
│   ├── error.rs            # HighlightError enum with thiserror [NEW]
│   ├── syntect_highlighter.rs  # syntect integration for code highlighting
│   ├── markdown_formatter.rs   # pulldown-cmark integration
│   ├── themes.rs           # Theme management and defaults
│   ├── options.rs          # HighlightOptions, MarkdownOptions structs
│   ├── ansi.rs             # ANSI escape code helper [NEW]
│   ├── error_annotations.rs    # Error highlighting ("red squigglies")
│   └── code_context.rs     # Context-aware code extraction (function/method scope)
├── output.rs               # Modified to use highlighting module
└── colorize.rs             # Deprecated (replaced by syntect)
```

### Component Diagram

```
┌────────────────────────────────────────────────────────────┐
│                    ta source (CLI)                         │
└───────────────────────┬────────────────────────────────────┘
                        │
                        v
┌────────────────────────────────────────────────────────────┐
│              OutputFormatter (lib/src/output.rs)           │
│  - format_type_errors() [MODIFIED]                         │
│  - format_symbols() [MODIFIED]                             │
└───────────────────────┬────────────────────────────────────┘
                        │
                        v
┌────────────────────────────────────────────────────────────┐
│          Highlighting Module (lib/src/highlighting/)       │
│                                                            │
│  ┌──────────────────────────────────────────────────┐     │
│  │  highlight_code(code, options) -> Result         │     │
│  │  format_markdown(text, options) -> Result        │     │
│  └──────────────────────┬───────────────────────────┘     │
│                         │                                  │
│         ┌───────────────┼───────────────┐                 │
│         v               v               v                 │
│  ┌──────────┐  ┌────────────┐  ┌──────────────┐          │
│  │ syntect  │  │ pulldown-  │  │    Error     │          │
│  │Highlight │  │   cmark    │  │  Annotations │          │
│  │   er     │  │ Parser     │  │              │          │
│  └──────────┘  └────────────┘  └──────────────┘          │
│         │               │               │                 │
│         v               v               v                 │
│  ┌──────────────────────────────────────────────────┐     │
│  │         Output Renderers                         │     │
│  │  - ConsoleRenderer (ANSI escape codes)           │     │
│  │  - HtmlRenderer (semantic HTML + CSS classes)    │     │
│  │  - JsonRenderer (structured data)                │     │
│  └──────────────────────────────────────────────────┘     │
└────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **CLI Input** → `ta source` receives analysis request
2. **Analysis** → OXC analyzer detects type errors with spans
3. **Context Extraction** → `code_context.rs` extracts function/method scope around error
4. **Highlighting** → `highlight_code()` applies syntect theme to extracted code
5. **Error Annotation** → `error_annotations.rs` adds "red squigglies" at error spans
6. **Output Rendering** → Appropriate renderer (Console/HTML/JSON) produces final output
7. **Display** → Formatted output to STDOUT

---

## Test Fixtures Strategy

**Location:** `cli/tests/fixtures/highlighting/`

All test fixtures must be created in Phase 1 to support TDD workflow throughout implementation.

### Required TypeScript Fixtures

1. **basic_error.ts** - Single type error in simple function
2. **multiple_errors.ts** - 5+ errors across different scopes
3. **nested_scope.ts** - Error in class method within namespace
4. **long_function.ts** - 30-line function to test truncation logic
5. **edge_cases.ts** - Errors at line 1, EOF, in comments
6. **unicode.ts** - Emoji, Chinese characters, RTL text
7. **multiline_span.ts** - Error spanning 3+ lines
8. **malformed.ts** - Syntax errors for graceful degradation

### Expected Output Files

Each fixture must have corresponding expected output files:
- `basic_error.expected.console` - Console output with ANSI codes
- `basic_error.expected.html` - HTML output
- `basic_error.expected.json` - JSON output

These files will be used for snapshot testing with `insta`.

---

## Phases

### Phase 1: Core Highlighting Infrastructure

**Principal Owner:** Rust Developer

**Goal:** Set up syntect integration, error types, theme management, ANSI helpers, and basic code highlighting for Console and HTML outputs. **Validate performance assumptions with early benchmarking.**

**Dependencies:** None

**Blast Radius:** `cargo test --lib highlighting`

**Coverage Target:** `syntect_highlighter.rs` - 90%

**Deliverables:**
- File `lib/src/highlighting/mod.rs` exists with public API (>150 lines)
- File `lib/src/highlighting/error.rs` exists with error types (>80 lines) **[REVIEW CHANGE]**
- File `lib/src/highlighting/syntect_highlighter.rs` exists with >300 lines
- File `lib/src/highlighting/themes.rs` exists with >150 lines
- File `lib/src/highlighting/options.rs` exists with >150 lines **[REVIEW CHANGE: Added Default + builders]**
- File `lib/src/highlighting/ansi.rs` exists with >100 lines **[REVIEW CHANGE]**
- File `lib/benches/highlighting.rs` exists with benchmark **[REVIEW CHANGE: Moved from Phase 8]**
- Test fixtures created in `cli/tests/fixtures/highlighting/` **[REVIEW CHANGE: Moved from Phase 8]**
- Add dependencies to `lib/Cargo.toml`:
  - `syntect = { version = "5.2", default-features = false, features = ["default-syntaxes", "default-themes", "html", "parsing"] }`
- Default themes configured (light: "Solarized (light)", dark: "base16-ocean.dark")

**TDD Approach:** **[REVIEW CHANGE]**
1. Write failing unit tests first defining expected behavior for `highlight_code()`
2. Implement minimal code to pass tests incrementally
3. Run `cargo test --lib highlighting` after each change
4. Refactor when tests are green
5. Add integration tests for error type handling

**Technical Details:**

**1. Error Types (CRITICAL - Define First):**
```rust
// lib/src/highlighting/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HighlightError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Theme '{name}' not found")]
    ThemeNotFound { name: String },

    #[error("Failed to load theme from file: {source}")]
    ThemeLoadError {
        #[from]
        source: std::io::Error,
    },

    #[error("Invalid code span: line {line}, column {column}")]
    InvalidSpan { line: usize, column: usize },

    #[error("Code block exceeds maximum size ({size} lines > {max} lines)")]
    CodeBlockTooLarge { size: usize, max: usize },

    #[error("Syntax highlighting failed: {0}")]
    SyntectError(String),
}

pub type Result<T> = std::result::Result<T, HighlightError>;
```

**2. ANSI Escape Code Helper:**
```rust
// lib/src/highlighting/ansi.rs
pub struct AnsiBuilder {
    codes: Vec<String>,
}

impl AnsiBuilder {
    pub fn new() -> Self { Self { codes: vec![] } }

    pub fn fg_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.codes.push(format!("38;2;{};{};{}", r, g, b));
        self
    }

    pub fn bg_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.codes.push(format!("48;2;{};{};{}", r, g, b));
        self
    }

    pub fn bold(mut self) -> Self {
        self.codes.push("1".to_string());
        self
    }

    pub fn italic(mut self) -> Self {
        self.codes.push("3".to_string());
        self
    }

    pub fn underline(mut self) -> Self {
        self.codes.push("4".to_string());
        self
    }

    pub fn build(&self) -> String {
        if self.codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", self.codes.join(";"))
        }
    }

    pub const RESET: &'static str = "\x1b[0m";
}

// Terminal capability detection
pub fn detect_terminal_capabilities() -> TerminalCapabilities {
    use std::env;

    let colorterm = env::var("COLORTERM").ok();
    let term = env::var("TERM").ok();

    if colorterm.as_deref() == Some("truecolor") ||
       colorterm.as_deref() == Some("24bit") {
        TerminalCapabilities::TrueColor
    } else if term.as_ref().map(|s| s.contains("256")).unwrap_or(false) {
        TerminalCapabilities::Color256
    } else {
        TerminalCapabilities::Basic16
    }
}

pub enum TerminalCapabilities {
    TrueColor,  // 24-bit RGB
    Color256,   // 8-bit palette
    Basic16,    // Basic ANSI colors
}
```

**3. Options Structs with Default + Builder:**
```rust
// lib/src/highlighting/options.rs
use crate::output::OutputFormat;
use crate::highlighting::error_annotations::ErrorAnnotation;

#[derive(Debug, Clone)]
pub struct HighlightOptions {
    pub language: String,
    pub light_theme: Option<String>,
    pub dark_theme: Option<String>,
    pub show_line_numbers: bool,
    pub error_spans: Vec<ErrorAnnotation>,
    pub output_format: OutputFormat,
}

impl Default for HighlightOptions {
    fn default() -> Self {
        Self {
            language: "typescript".to_string(),
            light_theme: None,  // Will use "Solarized (light)"
            dark_theme: None,   // Will use "base16-ocean.dark"
            show_line_numbers: false,
            error_spans: Vec::new(),
            output_format: OutputFormat::Console,
        }
    }
}

impl HighlightOptions {
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
            ..Default::default()
        }
    }

    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        let theme = theme.into();
        self.light_theme = Some(theme.clone());
        self.dark_theme = Some(theme);
        self
    }

    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn with_errors(mut self, errors: Vec<ErrorAnnotation>) -> Self {
        self.error_spans = errors;
        self
    }

    pub fn for_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }
}

#[derive(Debug, Clone)]
pub struct MarkdownOptions {
    pub code_light_theme: Option<String>,
    pub code_dark_theme: Option<String>,
    pub show_line_numbers: bool,
    pub output_format: OutputFormat,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            code_light_theme: None,
            code_dark_theme: None,
            show_line_numbers: false,
            output_format: OutputFormat::Console,
        }
    }
}
```

**4. Public API with Result Types:**
```rust
// lib/src/highlighting/mod.rs
use crate::highlighting::error::Result;

/// Highlights code with syntax highlighting and optional error annotations.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::{highlight_code, HighlightOptions};
///
/// let code = "function add(a: number) { return a + 1; }";
/// let result = highlight_code(code, HighlightOptions::default())?;
/// # Ok::<(), ta_lib::highlighting::HighlightError>(())
/// ```
pub fn highlight_code(
    code: &str,
    options: HighlightOptions
) -> Result<HighlightedCode> {
    // Implementation
}

pub fn format_markdown(
    text: &str,
    options: MarkdownOptions
) -> Result<FormattedMarkdown> {
    // Implementation
}
```

**5. Output Types (Separate Data from Rendering):**
```rust
// Internal representation
#[derive(Debug, Clone, serde::Serialize)]
pub struct HighlightedCode {
    pub segments: Vec<HighlightSegment>,
    pub errors: Vec<ErrorAnnotation>,
    pub line_count: usize,
    pub language: String,
    pub theme: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HighlightSegment {
    pub text: String,
    pub style: Style,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

// Rendering methods
impl HighlightedCode {
    pub fn render_console(&self) -> String { /* ANSI codes */ }
    pub fn render_html(&self) -> String { /* HTML */ }
    pub fn to_json(&self) -> serde_json::Value { /* JSON */ }
}
```

**6. Benchmarking (Early Performance Validation):**
```rust
// lib/benches/highlighting.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ta_lib::highlighting::{highlight_code, HighlightOptions};

fn benchmark_highlighting(c: &mut Criterion) {
    let code_1000_lines = /* Generate 1000-line TypeScript code */;

    c.bench_function("highlight 1000 lines", |b| {
        b.iter(|| {
            highlight_code(black_box(&code_1000_lines), HighlightOptions::default())
        })
    });
}

criterion_group!(benches, benchmark_highlighting);
criterion_main!(benches);
```

**Acceptance Criteria:**
- [ ] `grep "pub enum HighlightError" lib/src/highlighting/error.rs` succeeds
- [ ] All error variants have descriptive `#[error()]` messages
- [ ] `grep "pub fn highlight_code" lib/src/highlighting/mod.rs` succeeds
- [ ] All highlighting functions return `Result<T, HighlightError>`
- [ ] `grep "impl Default for HighlightOptions" lib/src/highlighting/options.rs` succeeds
- [ ] Builder methods exist and are tested
- [ ] `grep "pub struct AnsiBuilder" lib/src/highlighting/ansi.rs` succeeds
- [ ] `AnsiBuilder` supports fg/bg RGB, bold, italic, underline
- [ ] Terminal capability detection works (`detect_terminal_capabilities()`)
- [ ] `grep "syntect =" lib/Cargo.toml` succeeds
- [ ] `cargo test --lib highlighting` runs 20+ tests (increased from 15)
- [ ] Console output contains ANSI 24-bit color codes (`\x1b[38;2;`)
- [ ] HTML output contains `<span>` elements with style
- [ ] Default themes load without errors
- [ ] TypeScript language detection works
- [ ] `cargo bench` runs highlighting benchmark
- [ ] Benchmark shows <10ms for 1000 lines of TypeScript
- [ ] Test fixtures exist in `cli/tests/fixtures/highlighting/`
- [ ] Code coverage ≥90% for `syntect_highlighter.rs` (`cargo tarpaulin --lib`)
- [ ] All new tests pass
- [ ] Doc tests compile and run successfully

---

### Phase 2: Error Annotation System

**Principal Owner:** Rust Developer

**Goal:** Implement error highlighting with visual indicators ("red squigglies") and error messages. Use `Span` as single source of truth for error positions.

**Dependencies:** Phase 1 (highlighting infrastructure and error types must exist)

**Blast Radius:** `cargo test --lib highlighting`

**Coverage Target:** `error_annotations.rs` - 95%

**Deliverables:**
- File `lib/src/highlighting/error_annotations.rs` exists with >200 lines
- Error annotation types defined with `Span` as source of truth **[REVIEW CHANGE]**
- Console renderer shows errors with underlines
- HTML renderer includes popovers for error descriptions
- Serde derives for JSON output **[REVIEW CHANGE]**

**TDD Approach:**
1. Write failing tests for `ErrorAnnotation` creation and validation
2. Implement `ErrorAnnotation` struct with Span-based design
3. Write tests for console error rendering (ANSI underlines)
4. Implement console error rendering
5. Write tests for HTML popover generation
6. Implement HTML popover rendering
7. Run `cargo test error_annotations` after each change
8. Refactor when tests are green

**Technical Details:**

**1. ErrorAnnotation (Revised Design):**
```rust
// lib/src/highlighting/error_annotations.rs
use oxc_span::Span;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorAnnotation {
    span: Span,  // Private - single source of truth
    message: String,
    severity: ErrorSeverity,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

impl ErrorAnnotation {
    pub fn new(span: Span, message: String, severity: ErrorSeverity) -> Self {
        Self { span, message, severity }
    }

    pub fn span(&self) -> Span { self.span }
    pub fn message(&self) -> &str { &self.message }
    pub fn severity(&self) -> ErrorSeverity { self.severity }

    // Computed properties (derive from span + source text)
    pub fn line(&self, source: &str) -> usize {
        // Compute line from span.start byte offset
        source[..span.start as usize].chars().filter(|&c| c == '\n').count() + 1
    }

    pub fn column(&self, source: &str) -> usize {
        // Compute column from span.start
        let line_start = source[..span.start as usize]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        source[line_start..span.start as usize].chars().count() + 1
    }

    pub fn end_line(&self, source: &str) -> usize {
        source[..span.end as usize].chars().filter(|&c| c == '\n').count() + 1
    }

    pub fn end_column(&self, source: &str) -> usize {
        let line_start = source[..span.end as usize]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        source[line_start..span.end as usize].chars().count() + 1
    }
}
```

**2. Console Output - Red Underlines:**
```rust
// Add red underline beneath error spans
pub fn render_error_console(code: &str, annotation: &ErrorAnnotation) -> String {
    let line = annotation.line(code);
    let column = annotation.column(code);

    // Use AnsiBuilder for underline
    let underline = AnsiBuilder::new()
        .fg_rgb(255, 0, 0)
        .underline()
        .build();

    // Apply to error span
    // ...
}
```

**3. HTML Output - Popover API:**
```html
<span class="error-highlight" popovertarget="error-123" aria-describedby="error-123">
  <span class="squiggle" aria-label="Error">userId</span>
</span>
<div id="error-123" popover role="alert">
  <div class="error-message">Identifier 'userId' has already been declared</div>
</div>
```

**4. Multi-Error Support:**
- Support multiple `ErrorAnnotation` instances per code block
- Calculate visual column offsets accounting for syntax highlighting
- Handle overlapping error spans gracefully

**Acceptance Criteria:**
- [ ] `grep "pub struct ErrorAnnotation" lib/src/highlighting/error_annotations.rs` succeeds
- [ ] `ErrorAnnotation` uses `Span` as single source of truth (no redundant fields)
- [ ] Computed properties (`line()`, `column()`) work correctly
- [ ] `ErrorSeverity` has `#[non_exhaustive]` attribute
- [ ] `ErrorAnnotation` derives `Serialize` for JSON output
- [ ] Console output contains underline escape codes for errors
- [ ] HTML output contains `popovertarget` attributes
- [ ] HTML output includes ARIA labels (`aria-describedby`, `role="alert"`)
- [ ] Multiple errors in same code block render correctly
- [ ] Overlapping error spans handled gracefully
- [ ] Out-of-bounds spans return error (property tests)
- [ ] `cargo test error_annotations` runs 12+ tests (increased from 8)
- [ ] Code coverage ≥95% for `error_annotations.rs`
- [ ] All new tests pass

---

### Phase 3: Context-Aware Code Extraction

**Principal Owner:** Rust Developer

**Goal:** Extract context-aware code snippets around errors (function/method scope with smart truncation). Ensure all span operations are bounds-checked.

**Dependencies:** None (can run in parallel with Phase 1-2)

**Blast Radius:** `cargo test --lib highlighting`

**Coverage Target:** `code_context.rs` - 85%

**Deliverables:**
- File `lib/src/highlighting/code_context.rs` exists with >400 lines
- Integration with `lib/src/type_errors.rs` to extract source context
- Business logic for function/method/type scope detection
- Safety validation for span operations **[REVIEW CHANGE]**
- Serde derives for JSON output

**TDD Approach:**
1. Write failing tests for scope detection (function, method, module-level)
2. Implement scope detection using OXC semantic analysis
3. Write tests for truncation logic (<15 lines vs ≥15 lines)
4. Implement truncation with distinctive markers
5. Write property tests for bounds checking (no panics)
6. Implement safety validation for all span operations
7. Run `cargo test code_context` after each change
8. Refactor when tests are green

**Technical Details:**

**1. CodeContext with Safety Validation:**
```rust
// lib/src/highlighting/code_context.rs
use oxc_span::Span;
use oxc_semantic::Semantic;
use crate::highlighting::error::Result;

pub fn extract_code_context(
    source: &str,
    error_span: Span,
    semantic: &Semantic,
) -> Result<CodeContext> {
    // Validate span is within source bounds (CRITICAL SAFETY CHECK)
    if error_span.end as usize > source.len() {
        return Err(HighlightError::InvalidSpan {
            line: /* calculate */,
            column: /* calculate */,
        });
    }

    // Safe to proceed with extraction
    // ...
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CodeContext {
    pub full_code: String,
    pub display_code: String,  // With truncation
    pub scope_type: ScopeType,
    pub scope_name: String,
    pub truncation_info: Option<TruncationInfo>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq, Eq)]
pub enum ScopeType {
    Function,
    Method { class_name: String },
    TypeUtility,
    ModuleLevel,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TruncationInfo {
    pub original_line_count: usize,
    pub displayed_line_count: usize,
    pub truncated_sections: Vec<(usize, usize)>,  // (start_line, end_line)
}
```

**2. Business Logic (from feature requirements):**
- If in **function/method** AND <15 lines: show full definition
- If in **function/method** AND ≥15 lines:
  - Show first line (signature)
  - Show `┄┄┄ (10 lines omitted) ┄┄┄` truncation marker (distinctive)
  - Show 2 lines before error
  - Show error line
  - Show 2 lines after error
  - Show `┄┄┄ (5 lines omitted) ┄┄┄` truncation marker
  - Show closing bracket
- If in **type utility**: follow same logic as function/method
- All others:
  - `┄┄┄ (N lines omitted) ┄┄┄` marker
  - 2 lines before error
  - Error line
  - 2 lines after error
  - `┄┄┄ (N lines omitted) ┄┄┄` marker

**3. Edge Case Handling:**
- Error at start/end of file
- Multiple errors in same scope
- Anonymous functions, arrow functions
- Constructors, getters, setters
- Decorator errors
- Generic function errors
- Async function errors

**4. Property-Based Testing:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn extract_code_context_never_panics(
        source in "\\PC{0,10000}",
        start in 0usize..10000,
        end in 0usize..10000,
    ) {
        let span = Span::new(start as u32, end as u32);

        // Should either return Ok or Err, never panic
        let _ = extract_code_context(&source, span, &mock_semantic());
    }
}
```

**Acceptance Criteria:**
- [ ] File `lib/src/highlighting/code_context.rs` exists with >400 lines
- [ ] `grep "pub fn extract_code_context" lib/src/highlighting/code_context.rs` succeeds
- [ ] Out-of-bounds spans return `HighlightError::InvalidSpan`
- [ ] Property tests verify no panics with arbitrary spans
- [ ] All string slicing operations are bounds-checked
- [ ] Short functions (<15 lines) show full definition
- [ ] Long functions (≥15 lines) show truncated view with 2-line context
- [ ] Truncation markers are distinctive (`┄┄┄ (N lines omitted) ┄┄┄`)
- [ ] Method errors show `ClassName::methodName` scope
- [ ] Module-level errors show global scope
- [ ] Anonymous function errors handled
- [ ] Arrow function errors handled
- [ ] Constructor, getter, setter errors handled
- [ ] `CodeContext` derives `Serialize`
- [ ] `ScopeType` has `#[non_exhaustive]`
- [ ] `cargo test code_context` runs 18+ tests (increased from 12)
- [ ] Code coverage ≥85% for `code_context.rs`
- [ ] All new tests pass

---

### Phase 4: Markdown Parsing with Code Blocks

**Principal Owner:** Rust Developer

**Goal:** Implement `format_markdown()` using pulldown-cmark for markdown parsing with embedded code highlighting.

**Dependencies:** Phase 1 (highlighting infrastructure)

**Blast Radius:** `cargo test --lib highlighting`

**Coverage Target:** `markdown_formatter.rs` - 85%

**Deliverables:**
- File `lib/src/highlighting/markdown_formatter.rs` exists with >350 lines
- Add dependency to `lib/Cargo.toml`: `pulldown-cmark = "0.12"`
- Code block language indicators
- Code block titles (from info string)
- Visual separation between prose and code

**TDD Approach:**
1. Write failing tests for markdown parsing (basic code blocks)
2. Implement basic `pulldown_cmark::Parser` integration
3. Write tests for language detection from info string
4. Implement language and title extraction
5. Write tests for visual separators (Console output)
6. Implement visual separators with box-drawing characters
7. Write tests for edge cases (no language, unknown language, malformed fences)
8. Implement graceful fallbacks
9. Run `cargo test markdown_formatter` after each change
10. Refactor when tests are green

**Technical Details:**

**1. Markdown Parsing:**
```rust
// lib/src/highlighting/markdown_formatter.rs
use pulldown_cmark::{Parser, Event, Tag, CodeBlockKind};

pub fn format_markdown(
    text: &str,
    options: MarkdownOptions,
) -> Result<FormattedMarkdown> {
    let parser = Parser::new(text);

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                // Extract language and title
                let (lang, title) = parse_info_string(kind)?;
                // Highlight code block
            }
            // ...
        }
    }
}

fn parse_info_string(kind: CodeBlockKind) -> Result<(String, Option<String>)> {
    match kind {
        CodeBlockKind::Fenced(info) => {
            // Info string: "ts My Typescript Function"
            // Language: "ts"
            // Title: "My Typescript Function"
            let parts: Vec<&str> = info.split_whitespace().collect();
            let lang = parts.first().ok_or(HighlightError::UnsupportedLanguage("".to_string()))?;
            let title = if parts.len() > 1 {
                Some(parts[1..].join(" "))
            } else {
                None
            };
            Ok((lang.to_string(), title))
        }
        CodeBlockKind::Indented => Ok(("text".to_string(), None)),
    }
}
```

**2. Console Visual Separators:**
```
┌─ ts ──────────────────────────────── My Typescript Function ─┐
│ function MyFunction() {                                       │
│     console.log("hi")                                         │
│ }                                                             │
└───────────────────────────────────────────────────────────────┘
```

**3. HTML Code Block Header:**
```html
<div class="code-block">
  <div class="code-block__header">
    <span class="code-block__title">My Typescript Function</span>
    <span class="code-block__language" data-lang="ts">ts</span>
  </div>
  <pre><code class="language-ts">...</code></pre>
</div>
```

**4. Edge Cases:**
- Nested code blocks in lists/blockquotes
- Code fence without language (fallback to "text")
- Code fence with unknown language (fallback to plain text)
- Inline code (should NOT be highlighted, only fenced blocks)
- HTML in markdown (proper escaping)
- Malformed fences (missing closing fence - graceful degradation)

**Acceptance Criteria:**
- [ ] `grep "pulldown-cmark =" lib/Cargo.toml` succeeds
- [ ] `grep "pub fn format_markdown" lib/src/highlighting/markdown_formatter.rs` succeeds
- [ ] Code blocks are detected and highlighted
- [ ] Language indicators appear in output
- [ ] Code block titles are extracted from info string
- [ ] Visual separation between prose and code in Console output (box-drawing chars)
- [ ] Nested code blocks in lists handled
- [ ] Code fence without language defaults to "text"
- [ ] Unknown languages fall back to plain text
- [ ] Inline code is not highlighted
- [ ] HTML in markdown is escaped
- [ ] Malformed fences handled gracefully
- [ ] `cargo test markdown_formatter` runs 15+ tests (increased from 10)
- [ ] Code coverage ≥85% for `markdown_formatter.rs`
- [ ] All new tests pass

---

### Phase 5: HTML Interactive Features

**Principal Owner:** Rust Developer

**Goal:** Add HTML-specific interactive features (clipboard copy, hover animations). Ensure accessibility (ARIA, WCAG).

**Dependencies:** Phase 4 (markdown formatting must exist)

**Blast Radius:** `cargo test --lib highlighting` + manual browser testing

**Deliverables:**
- CSS animations for language indicator hover
- JavaScript for clipboard copy functionality
- Inline `<style>` and `<script>` tags in HTML output (optional)
- Documentation for external CSS/JS usage
- ARIA attributes for accessibility **[REVIEW CHANGE]**

**TDD Approach:**
1. Write tests for HTML structure with interactive elements
2. Implement CSS class generation
3. Write tests for ARIA attributes
4. Implement accessible HTML output
5. Manual browser testing for interactions
6. Document manual test checklist
7. Run `cargo test html_interactive` after changes

**Technical Details:**

**1. CSS Classes with Hover:**
```html
<span class="language copyable" data-lang="ts" onclick="copyCode(this)">ts</span>
```

```css
.language:hover {
  transform: scale(1.1);
  cursor: pointer;
  background-color: rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}
```

**2. JavaScript Clipboard:**
```javascript
function copyCode(elem) {
  const codeBlock = elem.closest('.code-block').querySelector('code');
  navigator.clipboard.writeText(codeBlock.textContent);

  // Show animation/feedback
  elem.classList.add('copied');
  setTimeout(() => elem.classList.remove('copied'), 1000);
}
```

**3. Accessibility (ARIA Attributes):**
```html
<div class="code-block" role="region" aria-label="Code example in TypeScript">
  <div class="code-block__header">
    <span class="code-block__language"
          role="button"
          aria-label="Copy code to clipboard"
          tabindex="0"
          data-lang="ts">ts</span>
  </div>
  <pre><code class="language-ts" aria-label="TypeScript code">...</code></pre>
</div>
```

**4. Optional Inline Styles:**
- Make CSS/JS injection optional via `HighlightOptions.inline_styles: bool`
- Provide external CSS/JS files for production use

**Acceptance Criteria:**
- [ ] HTML output contains `onclick` handlers for language indicators
- [ ] CSS classes for hover states are included
- [ ] JavaScript clipboard code is functional (manual browser test)
- [ ] Copied animation plays on click (manual browser test)
- [ ] Optional inline CSS/JS injection works
- [ ] ARIA labels on interactive elements (`role="button"`, `aria-label`)
- [ ] Keyboard navigation works (Tab, Enter to copy)
- [ ] High-contrast theme option available
- [ ] Color contrast ratios meet WCAG AA (4.5:1)
- [ ] Manual test checklist documented
- [ ] `cargo test` passes (no new unit tests required - visual features)

---

### Phase 6: Integration with `ta source` Command

**Principal Owner:** Rust Developer

**Goal:** Replace existing naive colorization in `ta source` with new highlighting system. Maintain backward compatibility with `TypeError.block` field.

**Dependencies:** Phase 1, Phase 2, Phase 3 (all core infrastructure)

**Blast Radius:** `cargo test --lib` (full library test suite)

**Coverage Target:** Modified modules - maintain existing coverage

**Deliverables:**
- Modified `lib/src/output.rs` to use `highlight_code()`
- Modified `lib/src/visitors/type_error_visitor.rs` to include source code extraction
- Modified `cli/src/commands/source.rs` to populate error annotations
- Deprecate `lib/src/colorize.rs` (mark as deprecated, remove in future)
- Backward compatibility maintained for `TypeError.block`

**TDD Approach:**
1. Write integration tests for new `ta source` output format
2. Implement `TypeError` model changes with backward compatibility
3. Write tests for `OutputFormatter` changes
4. Implement new formatting logic using `highlight_code()`
5. Write regression tests for JSON output compatibility
6. Run `cargo test --lib` after each change
7. Verify backward compatibility
8. Refactor deprecated code

**Technical Details:**

**1. TypeError Model (Backward Compatible):**
```rust
// lib/src/models.rs
#[derive(Debug, Clone, Serialize)]
pub struct TypeError {
    pub id: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub scope: String,

    // Legacy field - kept for backward compatibility
    // Contains plain text snippet as before
    pub block: String,

    // New field - context-aware code extraction
    // Will be populated by new highlighting system
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_code: Option<SourceCode>,

    #[serde(serialize_with = "span_serializer::serialize")]
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceCode {
    pub full_code: String,      // Full function/method/type
    pub display_code: String,   // With truncation applied
    pub scope_type: ScopeType,
    pub scope_name: String,
}
```

**2. OutputFormatter Integration:**
```rust
// lib/src/output.rs
use crate::highlighting::{highlight_code, HighlightOptions, ErrorAnnotation, ErrorSeverity};
use color_eyre::eyre::{Result, WrapErr};

fn format_type_errors_console(errors: &[TypeError]) -> String {
    let mut output = String::new();

    for error in errors {
        // Legacy: Still populate block field
        output.push_str(&format!("[{}] in {}\n", error.id, error.scope));

        // New: Use source_code if available
        if let Some(source) = &error.source_code {
            let highlighted = highlight_code(
                &source.display_code,
                HighlightOptions::new("typescript")
                    .with_errors(vec![ErrorAnnotation::new(
                        error.span,
                        error.message.clone(),
                        ErrorSeverity::Error,
                    )])
                    .with_line_numbers(true)
                    .for_format(OutputFormat::Console)
            )
            .wrap_err_with(|| format!("Failed to highlight code in {}", error.file))
            .unwrap_or_else(|_| {
                // Fallback to plain text
                source.display_code.clone()
            });

            output.push_str(&highlighted.render_console());
        } else {
            // Fallback to legacy block
            output.push_str(&format!("  {}\n", error.block));
        }

        output.push('\n');
    }

    output
}
```

**3. Deprecation of colorize.rs:**
```rust
// lib/src/colorize.rs
#![deprecated(since = "0.2.0", note = "Use highlighting module instead")]

// Keep existing code for now to avoid breaking changes
// Will be removed in 1.0.0
```

**Acceptance Criteria:**
- [ ] `ta source cli/tests/fixtures` produces highlighted output
- [ ] Error spans show "red squigglies" in console (via ANSI underlines)
- [ ] Function/method context is correctly extracted
- [ ] Line numbers appear in output
- [ ] HTML output works with `--format html`
- [ ] JSON output maintains backward compatibility (includes `block` field)
- [ ] `TypeError.source_code` is populated for new errors
- [ ] `TypeError.block` is still populated for legacy compatibility
- [ ] `cargo test --lib` passes with no warnings
- [ ] Integration tests verify all three output formats
- [ ] Regression tests confirm JSON structure unchanged
- [ ] Performance test: highlighting <10ms for 1000-line file
- [ ] Code coverage maintained for modified modules
- [ ] `colorize.rs` marked as `#[deprecated]`
- [ ] color-eyre `.wrap_err()` used for error context
- [ ] All new tests pass

---

### Phase 7: Theme Customization and Configuration

**Principal Owner:** Schema Architect

**Goal:** Implement theme loading from external files and configuration options. Add type-safe theme selection.

**Dependencies:** Phase 1 (theme infrastructure and error types must exist)

**Blast Radius:** `cargo test --lib highlighting::themes`

**Coverage Target:** `themes.rs` - 80%

**Deliverables:**
- Theme loading from `.tmTheme` files
- CLI arguments for theme selection (`--theme`, `--light-theme`, `--dark-theme`)
- Environment variable support (`TA_THEME`, `TA_LIGHT_THEME`, `TA_DARK_THEME`)
- Theme validation and error handling
- Type-safe theme abstraction **[REVIEW CHANGE]**

**TDD Approach:**
1. Write tests for built-in theme loading
2. Implement `BuiltinTheme` enum and loading logic
3. Write tests for custom theme file loading
4. Implement `.tmTheme` file parsing with validation
5. Write tests for CLI argument parsing
6. Implement CLI integration
7. Write tests for theme fallback behavior
8. Run `cargo test themes` after each change
9. Refactor when tests are green

**Technical Details:**

**1. Type-Safe Theme Abstraction:**
```rust
// lib/src/highlighting/themes.rs
use std::path::{Path, PathBuf};
use crate::highlighting::error::Result;

#[derive(Debug, Clone)]
pub enum ThemeSource {
    Builtin(BuiltinTheme),
    Custom(PathBuf),
}

#[derive(Debug, Clone, Copy)]
pub enum BuiltinTheme {
    SolarizedLight,
    Base16OceanDark,
    MonokaiExtended,
    Zenburn,
    Dracula,
    GruvboxDark,
    GruvboxLight,
}

impl BuiltinTheme {
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

    pub fn from_name(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "solarized-light" | "solarized (light)" => Ok(Self::SolarizedLight),
            "base16-ocean-dark" | "base16-ocean.dark" => Ok(Self::Base16OceanDark),
            "monokai-extended" | "monokai extended" => Ok(Self::MonokaiExtended),
            "zenburn" => Ok(Self::Zenburn),
            "dracula" => Ok(Self::Dracula),
            "gruvbox-dark" | "gruvbox dark" => Ok(Self::GruvboxDark),
            "gruvbox-light" | "gruvbox light" => Ok(Self::GruvboxLight),
            _ => Err(HighlightError::ThemeNotFound { name: name.to_string() }),
        }
    }
}

pub fn load_theme_from_file(path: &Path) -> Result<Theme> {
    // Validate path to prevent directory traversal
    let canonical = path.canonicalize()
        .map_err(|e| HighlightError::ThemeLoadError { source: e })?;

    if canonical.components().any(|c| c.as_os_str() == "..") {
        return Err(HighlightError::ThemeLoadError {
            source: std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Path traversal not allowed"
            ),
        });
    }

    // Load .tmTheme file
    // ...
}

pub fn list_available_themes() -> Vec<String> {
    BuiltinTheme::iter().map(|t| t.as_str().to_string()).collect()
}

pub fn get_theme(name: &str) -> Result<&'static Theme> {
    // Try built-in first, then custom
    // ...
}
```

**2. CLI Arguments:**
```rust
// cli/src/main.rs
#[arg(long, env = "TA_THEME")]
pub theme: Option<String>,

#[arg(long, env = "TA_LIGHT_THEME")]
pub light_theme: Option<String>,

#[arg(long, env = "TA_DARK_THEME")]
pub dark_theme: Option<String>,
```

**3. Theme Validation:**
- Support both built-in syntect themes and custom `.tmTheme` files
- Graceful fallback to defaults if theme not found
- Clear error messages with suggestions for similar theme names

**Acceptance Criteria:**
- [ ] `ThemeSource` enum exists with `Builtin` and `Custom` variants
- [ ] `BuiltinTheme` enum covers common themes
- [ ] `grep "pub enum BuiltinTheme" lib/src/highlighting/themes.rs` succeeds
- [ ] `ta source --theme "Monokai Extended"` uses specified theme
- [ ] `TA_THEME=zenburn ta source` respects environment variable
- [ ] Invalid theme names show helpful error message with suggestions
- [ ] Custom `.tmTheme` file loading works
- [ ] Path traversal attempts are rejected
- [ ] `ta source --list-themes` command lists available themes
- [ ] Theme fallback to defaults works
- [ ] `cargo test themes` runs 8+ tests (increased from 6)
- [ ] Code coverage ≥80% for `themes.rs`
- [ ] All new tests pass

---

### Phase 8: Testing and Documentation

**Principal Owner:** Feature Tester (Rust)

**Goal:** Comprehensive test coverage and documentation for all highlighting features. Ensure TDD was followed throughout (tests should already exist from previous phases).

**Dependencies:** All previous phases

**Blast Radius:** `cargo test` (full test suite)

**Deliverables:**
- Unit tests for all highlighting modules (targeting ≥85% code coverage overall)
- Integration tests for `ta source` with highlighting
- Property-based tests (proptest) for edge cases
- Snapshot tests (insta) for output consistency
- Doc tests for all public APIs (100% coverage)
- README updates documenting new features
- API documentation (rustdoc) for public functions
- CI/CD integration (GitHub Actions) **[REVIEW CHANGE]**
- Manual test documentation for browser features

**Technical Details:**

**1. Test Coverage by Module (Final Verification):**
- `highlighting/mod.rs`: 95% (public API surface)
- `highlighting/syntect_highlighter.rs`: 90% (core functionality)
- `highlighting/error_annotations.rs`: 95% (critical error rendering)
- `highlighting/code_context.rs`: 85% (complex business logic)
- `highlighting/markdown_formatter.rs`: 85% (parsing logic)
- `highlighting/themes.rs`: 80% (theme loading)
- `highlighting/ansi.rs`: 90% (ANSI code generation)
- **Overall highlighting module:** ≥85%

**2. Unit Tests (Already Written in Previous Phases):**
- `#[cfg(test)] mod tests` blocks in each module
- Test theme loading, language detection, error annotation positioning
- Mock OXC `Semantic` and `Span` for code context tests

**3. Integration Tests:**
```rust
// cli/tests/test_source_highlighting.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_ta_source_with_highlighting() {
    let mut cmd = Command::cargo_bin("ta").unwrap();

    cmd.arg("source")
       .arg("tests/fixtures/highlighting/basic_error.ts")
       .arg("--format")
       .arg("console");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;")) // ANSI 24-bit color
        .stdout(predicate::str::contains("Type 'string' is not assignable"));
}

#[test]
fn test_ta_source_html_output() {
    let mut cmd = Command::cargo_bin("ta").unwrap();

    cmd.arg("source")
       .arg("tests/fixtures/highlighting/basic_error.ts")
       .arg("--format")
       .arg("html");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("popovertarget"))
        .stdout(predicate::str::contains("<span class="));
}
```

**4. Property-Based Tests (Examples):**
```rust
proptest! {
    #[test]
    fn highlight_never_panics(code in "\\PC{0,1000}") {
        let _ = highlight_code(&code, HighlightOptions::default());
    }

    #[test]
    fn error_annotations_never_exceed_bounds(
        code in "\\PC{0,1000}",
        start in 0..1000usize,
        end in 0..1000usize,
    ) {
        let span = Span::new(start as u32, end as u32);
        let annotation = ErrorAnnotation::new(span, "test".to_string(), ErrorSeverity::Error);

        // Should never panic when accessing positions
        let _ = annotation.line(&code);
        let _ = annotation.column(&code);
    }
}
```

**5. Snapshot Tests:**
```rust
use insta::assert_snapshot;

#[test]
fn test_console_output_snapshot() {
    let code = include_str!("../fixtures/highlighting/basic_error.ts");
    let error = /* ... */;

    let options = HighlightOptions::new("typescript")
        .with_errors(vec![error])
        .with_line_numbers(true);

    let result = highlight_code(code, options).unwrap();

    assert_snapshot!("basic_error_console", result.render_console());
}

#[test]
fn test_html_output_snapshot() {
    let code = include_str!("../fixtures/highlighting/basic_error.ts");
    let error = /* ... */;

    let options = HighlightOptions::new("typescript")
        .with_errors(vec![error])
        .for_format(OutputFormat::Html);

    let result = highlight_code(code, options).unwrap();

    assert_snapshot!("basic_error_html", result.render_html());
}
```

**6. Doc Tests (100% Public API Coverage):**
All public functions must have working doc test examples:

```rust
/// Highlights code with syntax highlighting and optional error annotations.
///
/// # Examples
///
/// ```
/// use ta_lib::highlighting::{highlight_code, HighlightOptions};
///
/// let code = "function add(a: number) { return a + 1; }";
/// let result = highlight_code(code, HighlightOptions::default())?;
/// assert!(result.render_console().contains("\x1b["));
/// # Ok::<(), ta_lib::highlighting::HighlightError>(())
/// ```
///
/// # Errors
///
/// Returns `HighlightError::UnsupportedLanguage` if the language is not recognized.
pub fn highlight_code(code: &str, options: HighlightOptions) -> Result<HighlightedCode>
```

**7. Documentation Updates:**
- Update README with:
  - Code highlighting examples
  - Theme customization guide
  - Command-line usage with `--theme`
  - HTML output examples
- Add rustdoc comments to all public APIs
- Create examples in `examples/highlighting_demo.rs`
- Document manual browser test checklist for HTML features

**8. CI/CD Integration:**
```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check code coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --lib --out Lcov
      - name: Upload coverage
        uses: codecov/codecov-action@v2
      - name: Run benchmarks
        run: cargo bench --no-fail-fast
```

**9. Snapshot Test Management:**
```bash
# Update snapshots when output intentionally changes
cargo insta review

# Commit .snap files for review
git add **/*.snap

# CI runs this to catch unexpected changes
cargo insta test
```

**10. Manual Test Checklist for Browser Features:**
- [ ] Open HTML output in Chrome/Firefox/Safari
- [ ] Hover over language indicator shows scale animation
- [ ] Click language indicator copies code to clipboard
- [ ] Copied animation plays (1 second)
- [ ] Keyboard Tab navigation works
- [ ] Enter key on language indicator copies code
- [ ] Error popover appears on hover/click
- [ ] High-contrast mode readable
- [ ] Mobile viewport responsive

**Acceptance Criteria:**
- [ ] `cargo test` runs 100+ tests across all modules
- [ ] Code coverage ≥85% for highlighting modules (`cargo tarpaulin --lib`)
- [ ] Coverage report uploaded to codecov.io
- [ ] All property-based tests pass across 1000 iterations
- [ ] Snapshot tests generate consistent output
- [ ] Integration tests verify `ta source` highlighting for all formats
- [ ] Doc tests exist for 100% of public APIs
- [ ] All doc tests compile and run successfully
- [ ] README includes code highlighting examples
- [ ] `cargo doc --open` shows complete API documentation
- [ ] GitHub Actions workflow runs tests on CI
- [ ] Clippy lints pass with no warnings
- [ ] Benchmark regression detection configured
- [ ] Manual browser test checklist documented
- [ ] Manual browser tests pass (checked by human)
- [ ] Snapshot baselines committed to git
- [ ] All tests pass

---

## Blast Radius Analysis

| Phase | Test Command | Coverage Target | Rationale |
|-------|-------------|-----------------|-----------|
| Phase 1 | `cargo test --lib highlighting` | 90% syntect_highlighter.rs | New module with error types, ANSI helpers, benchmarks |
| Phase 2 | `cargo test --lib highlighting` | 95% error_annotations.rs | Extends highlighting module, critical rendering logic |
| Phase 3 | `cargo test --lib highlighting` | 85% code_context.rs | New code_context module, complex business logic |
| Phase 4 | `cargo test --lib highlighting` | 85% markdown_formatter.rs | New markdown_formatter module |
| Phase 5 | `cargo test --lib highlighting` + manual | N/A | HTML rendering changes, visual features |
| Phase 6 | `cargo test --lib` | Maintain existing | Touches output.rs and type_error_visitor.rs (wide impact) |
| Phase 7 | `cargo test --lib highlighting::themes` | 80% themes.rs | Theme loading only, narrow scope |
| Phase 8 | `cargo test` + `cargo tarpaulin` | ≥85% overall | Full suite validation, coverage verification |

---

## Review Summary

**Reviews Completed:** 2025-12-23
**Reviewers:** Rust Developer, Schema Architect, Feature Tester (Rust)
**Overall Assessment:** Approve with Changes (All mandatory changes incorporated)

### Key Changes from Review:

1. ✅ **Added Error Module to Phase 1** - `HighlightError` with thiserror
2. ✅ **Added Default + Builder Patterns** - All options structs ergonomic
3. ✅ **Added Serde Derives** - JSON-exposed types marked
4. ✅ **Revised ErrorAnnotation** - Span as single source of truth
5. ✅ **Added ANSI Helper Module** - Proper escape code generation
6. ✅ **Integrated TDD Throughout** - Each phase has TDD approach
7. ✅ **Defined Test Fixtures Early** - Phase 1 includes fixtures
8. ✅ **Per-Phase Coverage Targets** - Specific targets per module
9. ✅ **TypeError Backward Compatibility** - Documented migration strategy
10. ✅ **Moved Benchmarking to Phase 1** - Early performance validation
11. ✅ **Added color-eyre Integration** - Error context with `.wrap_err()`
12. ✅ **Terminal Capability Detection** - Graceful fallback for color support
13. ✅ **#[non_exhaustive] Annotations** - Future-proof enums
14. ✅ **Doc Tests Required** - 100% public API coverage

### Resolved Concerns:
- **Error handling design incomplete** → Error module in Phase 1
- **Data model couples data to rendering** → `HighlightedCode` struct with rendering methods
- **ANSI codes underspecified** → Full `AnsiBuilder` helper
- **Backward compatibility unclear** → `TypeError` keeps `block`, adds `source_code`
- **Testing delayed** → TDD integrated throughout all phases

---

## Cross-Cutting Concerns

### Testing Strategy

**Unit Tests:**
- `#[cfg(test)] mod tests` blocks in each module
- Test theme loading, language detection, error annotation positioning
- Mock OXC `Semantic` and `Span` for code context tests
- Written FIRST in TDD workflow (each phase)

**Integration Tests:**
- `cli/tests/test_source_highlighting.rs` - End-to-end `ta source` with real fixtures
- Fixtures in `cli/tests/fixtures/highlighting/` with known errors
- Test all three output formats (Console/HTML/JSON)
- Regression tests for JSON backward compatibility

**Property-Based Tests (proptest):**
- Fuzz code inputs to ensure no panics
- Test that error annotations never go out of bounds
- Verify markdown parsing handles malformed input gracefully
- 1000 iterations per property

**Snapshot Tests (insta):**
- Capture expected console output for regression testing
- Capture HTML structure for visual consistency
- Test theme application produces expected colors
- Managed with `cargo insta review`

**Doc Tests:**
- Examples in rustdoc comments demonstrate API usage
- Compile-checked documentation examples
- 100% coverage of public APIs

**Benchmarks:**
- `cargo bench` for syntax highlighting performance
- Target: <10ms per code block (<1000 lines)
- Regression detection with `criterion --baseline`

### Security Considerations

1. **HTML XSS Prevention:**
   - All user content (code, error messages) must be HTML-escaped
   - Use `html_escape::encode_text()` before rendering
   - Sanitize theme names to prevent path traversal

2. **File Path Safety:**
   - Validate `.tmTheme` file paths to prevent directory traversal
   - Use `std::path::canonicalize()` for path resolution
   - Reject paths containing `..` components

3. **Resource Limits:**
   - Limit code block size for highlighting (max 10,000 lines)
   - Prevent unbounded memory usage from large markdown documents
   - Return `HighlightError::CodeBlockTooLarge` for oversized input

### Performance Considerations

1. **Syntax Highlighting:**
   - Use `syntect::parsing::SyntaxSet::load_defaults_newlines()` - faster than `load_defaults_nonewlines()`
   - Cache `SyntaxSet` and `ThemeSet` in static `OnceCell` - load once per process
   - Target: <10ms per code block (<1000 lines), <100ms for 10,000 lines
   - **Validate early in Phase 1 with benchmarks**

2. **Markdown Parsing:**
   - `pulldown-cmark` is already very fast (100K+ events/sec)
   - Avoid unnecessary allocations when rendering prose (only highlight code blocks)

3. **Theme Loading:**
   - Lazy-load themes on first use
   - Cache loaded custom themes in memory
   - Built-in themes are compiled into binary (zero I/O)

4. **Error Annotation:**
   - Pre-calculate line offsets for O(1) lookup
   - Avoid quadratic algorithms when merging multiple errors

5. **Terminal Capability Detection:**
   - Detect once and cache result
   - Graceful degradation to 256-color or 16-color modes

### Project-Specific Concerns

**OXC Integration:**
- Extract source code using `Span` positions from OXC diagnostics
- Use `Semantic` analysis to find containing scope (function/method/type)
- Handle OXC's arena-allocated AST (all extracted code is owned `String`)

**Existing Colorization:**
- Deprecate `lib/src/colorize.rs` (naive keyword matching)
- Mark with `#[deprecated]` attribute
- Remove in 1.0.0 release

**Output Format Consistency:**
- Ensure Console, HTML, and JSON outputs have feature parity
- Console: ANSI 24-bit colors, underlines, box drawing characters
- HTML: Semantic markup, CSS classes, data-* attributes, popover API, ARIA labels
- JSON: Include both raw and rendered representations (optional)

**Error Context Extraction:**
- Reuse OXC `Semantic` from type error visitor (avoid double parsing)
- Handle edge cases: errors at EOF, in comments, in string literals
- Correctly identify scope for anonymous functions, arrow functions, class constructors

**Backward Compatibility:**
- `TypeError.block` field still populated (plain text snippet)
- `TypeError.source_code` field added as `Option<SourceCode>` (no breaking change)
- JSON output structure unchanged for existing consumers

---

## Parallelization Opportunities

```
Timeline:
─────────────────────────────────────────────────────►

Group A: ████████████████ (Phase 1 + Phase 3 in parallel)
                         │
Group B:                 └──██████████ (Phase 2 + Phase 4 in parallel)
                                     │
Group C:                             └──██████ (Phase 5 + Phase 6 in parallel)
                                              │
Group D:                                      └──████████ (Phase 7)
                                                        │
Group E:                                                └──██████████ (Phase 8)
```

### Parallel Execution Groups

| Group | Phases | Can Start After | Assignees |
|-------|--------|-----------------|-----------|
| A | 1, 3 | Plan approval | Rust Developer |
| B | 2, 4 | Group A complete | Rust Developer |
| C | 5, 6 | Group B complete | Rust Developer |
| D | 7 | Group C complete | Schema Architect |
| E | 8 | Group D complete | Feature Tester (Rust) |

**Rationale:**
- **Group A:** Phase 1 (highlighting infra + errors + ANSI) and Phase 3 (code context) have no dependencies
- **Group B:** Phase 2 (error annotations) needs Phase 1, Phase 4 (markdown) needs Phase 1
- **Group C:** Phase 5 (HTML features) extends Phase 4, Phase 6 (integration) needs Phases 1+2+3
- **Group D:** Phase 7 (theme config) extends Phase 1 but can wait for integration to stabilize
- **Group E:** Phase 8 (testing) verifies all features are complete (TDD means tests already exist)

### Synchronization Points

1. **After Group A:** Core highlighting API (`highlight_code`) and code extraction (`extract_code_context`) must be stable and tested
2. **After Group B:** Error annotations and markdown formatting must be functional
3. **After Group C:** `ta source` integration must be complete and working
4. **After Group D:** Theme customization must be tested
5. **Final:** All tests pass (≥85% coverage), documentation complete, CI green

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| syntect performance slower than expected | High | Benchmark in Phase 1 (not Phase 8), consider alternatives if >50ms for 1000 lines |
| pulldown-cmark event model incompatible | Medium | Prototype in Phase 4 immediately, fallback to manual parsing if needed |
| HTML Popover API not widely supported | Low | Feature-detect and provide fallback (tooltip or inline message) |
| ANSI escape codes not rendering | Medium | Detect terminal capabilities (`detect_terminal_capabilities()`), fallback to plain text |
| Theme file format incompatible | Low | Validate `.tmTheme` format early in Phase 7, clear error messages |
| Code context extraction fails | Medium | Extensive testing in Phase 3, fallback to full file if scope detection fails |
| Memory usage too high for large files | Medium | Implement line limits (10K max), return error for oversized input |
| Breaking backward compatibility | High | Keep `TypeError.block` populated, add `source_code` as optional field |

---

## Open Questions (Resolved)

All open questions from the original plan have been addressed:

✅ **Additional languages?** - Start with TypeScript, add others later (no change needed)
✅ **Auto-detect light/dark mode?** - Yes, use `detect_terminal_capabilities()` with CLI override
✅ **JSON include console representation?** - Make optional via flag (future enhancement)
✅ **Bundle themes in binary?** - Yes, syntect includes them, allow custom themes from disk
✅ **Multi-line error spans?** - Yes, use computed properties from Span
✅ **Custom CSS/JS for HTML?** - Yes, via optional `inline_styles` flag

---

## Implementation Parallelization Strategy

```
Phase 1 (Errors + Infra + ANSI)  ━━━━━━━━━━━━━━━━┓
                                              ┣━━► Phase 2 (Error Annotations) ━━┓
Phase 3 (Code Context)           ━━━━━━━━━━━━━━━━┛                                  ┃
                                                                                 ┣━━► Phase 5 (HTML Interactive) ━┓
                              ┌────────────────────────────────────────────────┘                                  ┃
                              │                                                                                   ┃
                              └━━► Phase 4 (Markdown Parsing) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
                                                                                                                   │
Phase 6 (ta source Integration) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
                                                                                                                   │
                                                                                                                   ├━━► Phase 7 (Theme Config) ━━► Phase 8 (Testing & Docs)
```

**Critical Path:** Phase 1 → Phase 2 → Phase 6 → Phase 7 → Phase 8

**Estimated Duration:**
- Sequential: 5-7 days
- With parallelization: 3-4 days

**Parallelizable Work:** Phase 3 can be developed independently and merged when ready for Phase 6

---

## Next Steps After Planning

1. ✅ **Review and Approval:** All reviews complete, mandatory changes incorporated
2. **Environment Setup:** Ensure `syntect` and `pulldown-cmark` dependencies compile
3. **Phase Execution:** Use `/execute-phase 1` to start implementation with TDD
4. **Continuous Testing:** Run `cargo test --lib highlighting` after each change
5. **Integration Validation:** Test `ta source` with real-world TypeScript projects
6. **Documentation:** Keep README and rustdoc updated throughout development
7. **User Feedback:** Gather feedback on output formats and iterate

---

**Status:** Reviewed - Ready for Implementation
**Review Date:** 2025-12-23
**Next Action:** Begin Phase 1 implementation with TDD workflow
