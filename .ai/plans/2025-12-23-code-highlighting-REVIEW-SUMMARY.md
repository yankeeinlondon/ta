# Code Highlighting Plan - Consolidated Review Summary

**Date:** 2025-12-23
**Plan:** .ai/plans/2025-12-23-code-highlighting.md
**Status:** APPROVED WITH CHANGES

---

## Review Summary

All three reviewers (Rust Developer, Schema Architect, Feature Tester) gave **"Approve with Changes"** assessments. The plan is fundamentally sound with excellent technology choices and clear phases, but requires several critical changes before implementation.

---

## Consensus Across All Reviews

### ✅ Unanimous Strengths

1. **Clear Module Structure** - Well-organized `lib/src/highlighting/` with single-responsibility modules
2. **Appropriate Technology** - syntect and pulldown-cmark are industry-standard choices
3. **Comprehensive Testing** - Good use of unit tests, integration tests, proptest, insta, and criterion
4. **Parallelization Strategy** - Correctly identifies independent phases
5. **Security Awareness** - XSS prevention, path traversal protection properly identified

### ⚠️ Critical Issues (All Reviewers Agree)

1. **Missing Error Module** - Error types not defined until Phase 7, should be in Phase 1
2. **API Returns Missing `Result`** - Functions return data directly instead of `Result<T, HighlightError>`
3. **Incomplete Type Design** - Options structs lack `Default` and builder patterns
4. **Testing Delayed** - TDD workflow should be integrated throughout, not just Phase 8

---

## Mandatory Changes (MUST FIX)

These changes are required before implementation begins:

### 1. Add Error Module to Phase 1 (CRITICAL)

**From:** Rust Developer, Schema Architect
**Impact:** High - Affects all subsequent phases

**Change:**

- Add file: `lib/src/highlighting/error.rs` (>80 lines)
- Define `HighlightError` enum with thiserror
- Include error types:
    - `UnsupportedLanguage(String)`
    - `ThemeNotFound { name: String }`
    - `ThemeLoadError { source: io::Error }`
    - `InvalidSpan { line: usize, column: usize }`
    - `CodeBlockTooLarge { size: usize, max: usize }`

**Example:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HighlightError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Theme '{name}' not found")]
    ThemeNotFound { name: String },

    #[error("Failed to load theme: {source}")]
    ThemeLoadError { #[from] source: std::io::Error },

    #[error("Invalid code span: line {line}, column {column}")]
    InvalidSpan { line: usize, column: usize },
}

pub type Result<T> = std::result::Result<T, HighlightError>;
```

**Update Phase 1 Acceptance Criteria:**

- [ ] `grep "pub enum HighlightError" lib/src/highlighting/error.rs` succeeds
- [ ] All error variants have descriptive `#[error()]` messages
- [ ] All highlighting functions return `Result<T, HighlightError>`

---

### 2. Add Default + Builder Pattern for Options Structs (CRITICAL)

**From:** Schema Architect
**Impact:** High - Affects API ergonomics

**Change:**

- Implement `Default` for `HighlightOptions` and `MarkdownOptions`
- Add builder methods for fluent API construction

**Example:**

```rust
impl Default for HighlightOptions {
    fn default() -> Self {
        Self {
            language: "typescript".to_string(),
            light_theme: None,
            dark_theme: None,
            show_line_numbers: false,
            error_spans: Vec::new(),
            output_format: OutputFormat::Console,
        }
    }
}

impl HighlightOptions {
    pub fn new(language: impl Into<String>) -> Self {
        Self { language: language.into(), ..Default::default() }
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
}
```

**Update Phase 1 Acceptance Criteria:**

- [ ] `grep "impl Default for HighlightOptions" lib/src/highlighting/options.rs` succeeds
- [ ] Builder methods exist and are tested
- [ ] Example usage in rustdoc shows ergonomic API

---

### 3. Add Serde Derives to All JSON-Exposed Types (HIGH)

**From:** Schema Architect
**Impact:** Medium - Required for JSON output

**Change:**

- Add `#[derive(Serialize)]` to all types that appear in JSON output

**Types requiring Serialize:**

- `HighlightedCode` (or its constituent parts)
- `ErrorAnnotation`
- `ErrorSeverity`
- `ScopeType`
- `CodeContext`

**Example:**

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorAnnotation {
    pub span: Span,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}
```

**Update Phase 1 Technical Details:**

- Specify which types derive `Serialize`
- Document JSON output schema

---

### 4. Revise ErrorAnnotation to Use Span as Source of Truth (HIGH)

**From:** Schema Architect
**Impact:** Medium - Prevents redundant fields and inconsistencies

**Change:**

- Remove redundant `line`, `column`, `end_line`, `end_column` fields
- Store only `Span` and compute line/column on demand

**Before:**

```rust
pub struct ErrorAnnotation {
    pub span: Span,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub message: String,
}
```

**After:**

```rust
pub struct ErrorAnnotation {
    span: Span,  // Private - single source of truth
    message: String,
    severity: ErrorSeverity,
}

impl ErrorAnnotation {
    pub fn new(span: Span, message: String, severity: ErrorSeverity) -> Self {
        Self { span, message, severity }
    }

    pub fn span(&self) -> Span { self.span }
    pub fn message(&self) -> &str { &self.message }
    pub fn severity(&self) -> ErrorSeverity { self.severity }

    // Computed properties (derive from span + source text)
    pub fn line(&self, source: &str) -> usize { /* compute */ }
    pub fn column(&self, source: &str) -> usize { /* compute */ }
}
```

**Update Phase 2 Technical Details:**

- Use Span as single source of truth
- Add computed property methods
- Update acceptance criteria

---

### 5. Add ANSI Escape Code Helper Module (HIGH)

**From:** Rust Developer
**Impact:** Medium - Required for correct Console output

**Change:**

- Add file: `lib/src/highlighting/ansi.rs` (>100 lines)
- Create `AnsiBuilder` for fluent ANSI code construction

**Example:**

```rust
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
```

**Update Phase 1 Deliverables:**

- File `lib/src/highlighting/ansi.rs` exists with >100 lines
- `AnsiBuilder` supports fg/bg RGB, bold, italic, underline
- Tests verify correct escape sequence generation

---

## Important Changes (SHOULD FIX)

### 6. Integrate TDD Workflow Throughout (Feature Tester)

**Change:** Add TDD approach to each phase (1-7), not just Phase 8

**Template to add to each phase:**

```markdown
**TDD Approach:**
1. Write failing unit tests first defining expected behavior
2. Implement minimal code to pass tests
3. Run `cargo test --lib [module]` after each change
4. Refactor when tests are green
5. Add integration tests if applicable
```

---

### 7. Define Test Fixtures Strategy Early (Feature Tester)

**Change:** Add test fixture specification to Phase 1, not Phase 8

**Required Fixtures (cli/tests/fixtures/highlighting/):**

- `basic_error.ts` - Single type error in simple function
- `multiple_errors.ts` - 5+ errors across different scopes
- `nested_scope.ts` - Error in class method within namespace
- `long_function.ts` - 30-line function to test truncation
- `edge_cases.ts` - Errors at line 1, EOF, in comments
- `unicode.ts` - Emoji, Chinese characters, RTL text
- `multiline_span.ts` - Error spanning 3+ lines
- `malformed.ts` - Syntax errors for graceful degradation

Each fixture must have corresponding `.expected.console`, `.expected.html`, and `.expected.json` snapshot files.

---

### 8. Add Per-Phase Coverage Targets (Feature Tester)

**Change:** Don't wait until Phase 8 for coverage measurement

**Targets:**

- Phase 1: `syntect_highlighter.rs` - 90% coverage
- Phase 2: `error_annotations.rs` - 95% coverage
- Phase 3: `code_context.rs` - 85% coverage
- Phase 4: `markdown_formatter.rs` - 85% coverage
- Phase 7: `themes.rs` - 80% coverage

**Measurement:** `cargo tarpaulin --lib`

---

### 9. Clarify TypeError Backward Compatibility (Rust Developer)

**Change:** Document migration strategy for `block` vs `source_code` fields

**Recommendation:**

```rust
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
    pub source_code: Option<SourceCode>,

    pub span: Span,
}

pub struct SourceCode {
    pub full_code: String,      // Full function/method/type
    pub display_code: String,   // With truncation applied
    pub scope_type: ScopeType,
    pub scope_name: String,
}
```

---

### 10. Move Benchmarking to Phase 1 (Rust Developer)

**Change:** Validate performance early, not in Phase 8

**Rationale:** If syntect is too slow, need to know before building on top of it

**Add to Phase 1:**

- Benchmark in `lib/benches/highlighting.rs`
- Target: <10ms for 1000 lines
- Acceptance criteria:
    - [ ] `cargo bench` runs highlighting benchmark
    - [ ] Benchmark shows <10ms for 1000 lines of TypeScript

---

## Additional Recommendations

### 11. Add color-eyre Integration (Rust Developer)

Per color-eyre skill guidelines, use `.wrap_err()` and `.with_help()` for context:

```rust
let highlighted = highlight_code(&source, options)
    .map_err(|e| eyre::Report::new(e))
    .wrap_err_with(|| format!("Failed to highlight code in {}", error.file))
    .with_help("Check that the file contains valid TypeScript syntax")?;
```

---

### 12. Add Terminal Capability Detection (Rust Developer)

Not all terminals support 24-bit color:

```rust
pub fn detect_terminal_capabilities() -> TerminalCapabilities {
    use std::env;

    let colorterm = env::var("COLORTERM").ok();
    if colorterm.as_deref() == Some("truecolor") ||
       colorterm.as_deref() == Some("24bit") {
        TerminalCapabilities::TrueColor
    } else {
        TerminalCapabilities::Color256  // Fallback
    }
}
```

---

### 13. Add #[non_exhaustive] for Future-Proofing (Schema Architect)

Allow adding enum variants without breaking changes:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}
```

---

### 14. Add Doc Tests for Public APIs (Feature Tester)

All public functions need working doc test examples:

```rust
/// Highlights code with syntax highlighting.
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
pub fn highlight_code(code: &str, options: HighlightOptions) -> Result<HighlightedCode>
```

---

## Reviewers' Overall Assessments

### Rust Developer
**Assessment:** Approve with Changes
**Key Concerns:** Error handling, data model design, ANSI codes, backward compatibility
**Top Priority:** Add error module and fix return types

### Schema Architect
**Assessment:** Approve with Changes
**Key Concerns:** Missing error types, options structs need defaults, ErrorAnnotation redundancy
**Top Priority:** Define error types and add Default implementations

### Feature Tester (Rust)
**Assessment:** Approve with Changes
**Key Concerns:** TDD integration, test fixtures, coverage targets
**Top Priority:** Integrate TDD throughout phases, define fixtures early

---

## Next Steps

1. **Update Plan** with mandatory changes (errors, defaults, serde, ANSI helper)
2. **Add TDD sections** to each phase
3. **Define test fixtures** in Phase 1
4. **Add per-phase coverage targets**
5. **Update acceptance criteria** with new requirements
6. **Mark plan as "Reviewed - Ready for Implementation"**

---

## Implementation Recommendation

**Start with:** Phase 1 (with all added modules) + Phase 3 in parallel

**Critical Path:** Phase 1 → Phase 2 → Phase 6 → Phase 7 → Phase 8

**Estimated Duration:**

- Sequential: 5-7 days
- With parallelization: 3-4 days

---

**Review Completed:** 2025-12-23
**Plan Status:** APPROVED WITH MANDATORY CHANGES REQUIRED
**Reviewers:** Rust Developer, Schema Architect, Feature Tester (Rust)
