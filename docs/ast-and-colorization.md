# AST Parsing and Colorization Design Document

## Overview

This document outlines the design for the core library (`/lib`) of the TypeScript Analyzer (TA) project. The library leverages the Oxidation Compiler (OXC) for high-performance AST parsing and semantic analysis, providing the foundation for all CLI features including type error detection, symbol analysis, dependency tracking, and file watching.

## Architecture

### Core Library Structure

The library is organized into focused modules, each handling a specific analysis domain:

```
lib/src/
├── lib.rs              # Main library entry point and public API
├── analyzer.rs         # Core analysis coordinator and file processing
├── type_errors.rs      # Type error detection and reporting
├── symbols.rs          # Exported symbol extraction and filtering
├── dependencies.rs     # File and symbol dependency analysis
├── tests.rs            # Type test detection in test files
├── watcher.rs          # File system watching and event handling
├── models.rs           # Shared data structures and types
├── output.rs           # Output formatting (console, HTML, JSON)
├── colorize.rs         # ANSI escape code and HTML colorization utilities
└── visitors/           # Custom AST visitor implementations
    ├── mod.rs
    ├── type_error_visitor.rs
    ├── symbol_visitor.rs
    └── dependency_visitor.rs
```

### Key Design Principles

1. **Performance First**: Leverage OXC's arena allocation and semantic analysis for maximum speed
2. **Parallel Processing**: Use Rayon for concurrent file analysis across CPU cores
3. **Memory Efficient**: Reuse allocators and minimize allocations in hot paths
4. **Composable**: Each analysis module can be used independently or combined
5. **Output Agnostic**: Core analysis logic separated from output formatting

## Core Components

### Analyzer (analyzer.rs)

The main coordinator that orchestrates analysis across multiple files:

```rust
pub struct Analyzer {
    allocator: Allocator,
    options: AnalysisOptions,
}

pub struct AnalysisOptions {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub parallel: bool,
    pub max_files: Option<usize>,
}

impl Analyzer {
    pub fn analyze_files(&self, files: &[PathBuf]) -> Result<AnalysisResult, AnalysisError> {
        // Coordinate parallel analysis of multiple files
    }

    pub fn analyze_single_file(&self, file: &Path) -> Result<FileAnalysis, AnalysisError> {
        // Analyze individual file with all extractors
    }
}
```

### Type Error Analysis (type_errors.rs)

Extracts type errors with rich context information:

```rust
#[derive(Serialize, Debug)]
pub struct TypeError {
    pub id: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub scope: String,  // e.g., "file.ts::functionName", "file.ts::Class:method"
    pub block: String,  // Plain text code block
    pub span: Span,
}

pub struct TypeErrorExtractor {
    allocator: &'a Allocator,
}

impl<'a> TypeErrorExtractor {
    pub fn extract(&self, source: &str, semantic: &Semantic) -> Vec<TypeError> {
        // Use semantic analysis to find type errors
        // Extract scope context and code blocks
    }
}
```

### Symbol Analysis (symbols.rs)

Extracts exported symbols with type information:

```rust
#[derive(Serialize, Debug)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,  // Function, Class, Interface, Type, etc.
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub exported: bool,
    pub parameters: Option<Vec<ParameterInfo>>,  // For functions
    pub properties: Option<Vec<PropertyInfo>>,   // For classes/interfaces
}

#[derive(Serialize, Debug)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Type,
    Variable,
    Enum,
}

pub struct SymbolExtractor {
    allocator: &'a Allocator,
}

impl<'a> SymbolExtractor {
    pub fn extract_exported_symbols(&self, files: &[PathBuf]) -> Vec<SymbolInfo> {
        // Scan files for exported symbols
        // Use semantic analysis for accurate type information
    }
}
```

### Dependency Analysis (dependencies.rs)

Analyzes file and symbol-level dependencies:

```rust
#[derive(Serialize, Debug)]
pub struct FileDependencies {
    pub file: String,
    pub repo_dependencies: Vec<String>,     // Files in same repo
    pub external_dependencies: Vec<String>, // External packages
}

#[derive(Serialize, Debug)]
pub struct SymbolDependencies {
    pub symbol: String,
    pub file: String,
    pub dependencies: Vec<SymbolDependency>,
}

#[derive(Serialize, Debug)]
pub struct SymbolDependency {
    pub name: String,
    pub scope: DependencyScope,  // Local, Repo, Module, External
    pub file: Option<String>,
}

pub enum DependencyScope {
    Local,   // Same file
    Repo,    // Different file in repo
    Module,  // Monorepo module
    External,// External package
}

pub struct DependencyExtractor {
    allocator: &'a Allocator,
    resolver: Option<Resolver>,  // For module resolution
}

impl<'a> DependencyExtractor {
    pub fn extract_file_dependencies(&self, file: &Path) -> FileDependencies {
        // Parse imports and analyze dependencies
    }

    pub fn extract_symbol_dependencies(&self, semantic: &Semantic) -> Vec<SymbolDependencies> {
        // Use symbol table to track symbol usage
    }
}
```

### Type Test Detection (tests.rs)

Identifies type tests in test files:

```rust
#[derive(Serialize, Debug)]
pub struct TypeTest {
    pub file: String,
    pub describe_block: String,
    pub test_name: String,
    pub line: usize,
    pub has_type_cases: bool,
    pub status: TestStatus,
}

#[derive(Serialize, Debug)]
pub enum TestStatus {
    Passing,
    Failing,
    NoTypeCases,
}

pub struct TestExtractor {
    allocator: &'a Allocator,
}

impl<'a> TestExtractor {
    pub fn extract_type_tests(&self, test_files: &[PathBuf]) -> Vec<TypeTest> {
        // Find describe/it blocks
        // Check for "type cases = [...]" patterns
        // Analyze test status
    }
}
```

## Colorization and Output System

### Console Colorization (colorize.rs)

Provides ANSI escape code utilities for terminal output:

```rust
pub struct ConsoleColorizer;

impl ConsoleColorizer {
    pub fn colorize_code_block(code: &str, language: &str) -> String {
        // Syntax highlighting with ANSI escape codes
        // Support for TypeScript/JavaScript keywords, strings, comments, etc.
    }

    pub fn highlight_error(error_span: &Span, source: &str) -> String {
        // Highlight error locations with red background
        // Show surrounding context
    }

    pub fn format_symbol_list(symbols: &[SymbolInfo]) -> String {
        // Colorized symbol listing with type indicators
    }
}

// ANSI color constants
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
```

### HTML Generation (colorize.rs)

Creates semantic HTML with metadata attributes:

```rust
pub struct HtmlColorizer;

impl HtmlColorizer {
    pub fn colorize_code_block(code: &str, language: &str) -> String {
        // Generate HTML with <span class="keyword">, <span class="string">, etc.
        // Include syntax highlighting
    }

    pub fn highlight_error(error: &TypeError, source: &str) -> String {
        // Create HTML with error highlighting
        // Add data attributes for tooling integration
    }

    pub fn format_symbol_table(symbols: &[SymbolInfo]) -> String {
        // HTML table with symbol information
        // Include data attributes for each symbol
    }
}

// HTML templates and classes
pub const HTML_HEADER: &str = r#"
<style>
.keyword { color: #0000ff; font-weight: bold; }
.string { color: #008000; }
.comment { color: #808080; font-style: italic; }
.error { background-color: #ffcccc; border: 1px solid #ff0000; }
.error-line { background-color: #ffe6e6; }
</style>
"#;
```

### Output Formatting (output.rs)

Unified output system supporting all three formats:

```rust
pub enum OutputFormat {
    Console,
    Html,
    Json,
}

pub struct OutputFormatter {
    colorizer: Colorizer,
}

impl OutputFormatter {
    pub fn format_type_errors(&self, errors: &[TypeError], format: OutputFormat) -> String {
        match format {
            OutputFormat::Console => self.format_type_errors_console(errors),
            OutputFormat::Html => self.format_type_errors_html(errors),
            OutputFormat::Json => self.format_type_errors_json(errors),
        }
    }

    pub fn format_symbols(&self, symbols: &[SymbolInfo], format: OutputFormat) -> String {
        match format {
            OutputFormat::Console => self.format_symbols_console(symbols),
            OutputFormat::Html => self.format_symbols_html(symbols),
            OutputFormat::Json => self.format_symbols_json(symbols),
        }
    }
}

// JSON output structures include formatted representations
#[derive(Serialize)]
pub struct JsonTypeErrorOutput {
    pub errors: Vec<TypeError>,
    pub console: String,
    pub html: String,
    pub summary: ErrorSummary,
}

#[derive(Serialize)]
pub struct JsonSymbolOutput {
    pub symbols: Vec<SymbolInfo>,
    pub console: String,
    pub html: String,
    pub summary: SymbolSummary,
}
```

## Visitor Pattern Implementation

### Custom Visitors (visitors/)

Specialized AST visitors for different analysis types:

```rust
// Base visitor trait
pub trait AnalysisVisitor<'a> {
    fn visit_program(&mut self, program: &Program<'a>) {
        walk::walk_program(self, program);
    }

    // Override specific visit methods as needed
    fn visit_function(&mut self, func: &Function<'a>) {}
    fn visit_class(&mut self, class: &Class<'a>) {}
    fn visit_ts_interface_declaration(&mut self, interface: &TSInterfaceDeclaration<'a>) {}
}

// Type error visitor
pub struct TypeErrorVisitor<'a> {
    pub errors: Vec<TypeError>,
    pub source: &'a str,
    pub semantic: &'a Semantic,
}

impl<'a> AnalysisVisitor<'a> for TypeErrorVisitor<'a> {
    fn visit_ts_interface_declaration(&mut self, interface: &TSInterfaceDeclaration<'a>) {
        // Check for interface-specific type errors
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        // Check variable type annotations
    }
}

// Symbol visitor
pub struct SymbolVisitor<'a> {
    pub symbols: Vec<SymbolInfo>,
    pub exported_only: bool,
}

impl<'a> AnalysisVisitor<'a> for SymbolVisitor<'a> {
    fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        // Extract exported symbols
    }

    fn visit_function(&mut self, func: &Function<'a>) {
        if self.exported_only && !is_exported(func) {
            return;
        }
        // Extract function symbol info
    }
}
```

## File Watching (watcher.rs)

Implements file system monitoring with event-driven analysis:

```rust
#[derive(Serialize, Debug)]
pub enum WatchEvent {
    SourceFileChanged { file: String, content: String },
    SourceFileCreated { file: String },
    SourceFileRemoved { file: String },
    SymbolRenamed { old_name: String, new_name: String, file: String },
    SymbolAdded { name: String, kind: SymbolKind, file: String },
    SymbolRemoved { name: String, file: String },
    ModuleDepChanged { file: String },
    ExternalDepChanged { package: String },
    TestStatusChanged { file: String, test: String, status: TestStatus },
    NewFailingTest { file: String, test: String },
    TestFixed { file: String, test: String },
    NewTestAdded { file: String, test: String },
}

pub struct FileWatcher {
    analyzer: Analyzer,
    handlers: Vec<Box<dyn WatchHandler>>,
}

#[async_trait]
pub trait WatchHandler: Send + Sync {
    async fn handle_event(&self, event: &WatchEvent) -> Result<(), WatchError>;
}

impl FileWatcher {
    pub async fn watch(&self, paths: &[PathBuf]) -> Result<(), WatchError> {
        // Set up file system watching
        // Analyze initial state
        // Monitor for changes and trigger events
    }
}
```

## Performance Optimizations

### Parallel Processing

```rust
pub fn analyze_files_parallel(files: &[PathBuf]) -> Vec<FileAnalysis> {
    files.par_iter()
        .filter_map(|file| {
            let allocator = Allocator::default();  // Thread-local allocator
            let source = std::fs::read_to_string(file).ok()?;
            let analysis = analyze_single_file(&allocator, &source, file)?;
            Some(analysis)
        })
        .collect()
}
```

### Memory Management

- Reuse `Allocator` instances within threads
- Use arena allocation for AST nodes
- Minimize string allocations in hot paths
- Stream large outputs instead of buffering

### Caching

- Cache parsed ASTs for files that haven't changed
- Cache symbol tables for repeated analysis
- Use incremental analysis for watch mode

## Error Handling

Comprehensive error types for different failure modes:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Semantic analysis error: {0}")]
    SemanticError(String),

    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid source type for file: {0}")]
    InvalidSourceType(String),

    #[error("Dependency resolution failed: {0}")]
    DependencyResolutionError(String),
}
```

## Testing Strategy

Unit tests for individual components:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_error_extraction() {
        let source = r#"
            function add(a: number, b: string): number {
                return a + b;  // Type error: can't add number + string
            }
        "#;

        let analyzer = Analyzer::new();
        let result = analyzer.analyze_source(source, "test.ts").unwrap();

        assert_eq!(result.type_errors.len(), 1);
        assert!(result.type_errors[0].message.contains("number + string"));
    }

    #[test]
    fn test_symbol_extraction() {
        let source = r#"
            export function greet(name: string): string {
                return `Hello, ${name}!`;
            }

            export class User {
                constructor(public name: string) {}
            }
        "#;

        let analyzer = Analyzer::new();
        let result = analyzer.analyze_source(source, "test.ts").unwrap();

        assert_eq!(result.symbols.len(), 2);
        // Verify symbol details
    }
}
```

## Integration with CLI

The library provides a clean API for the CLI layer:

```rust
// CLI integration example
pub fn run_source_analysis(filter: &str, format: OutputFormat) -> Result<String, AnalysisError> {
    let analyzer = Analyzer::new(AnalysisOptions::default());
    let files = find_files_by_filter(filter)?;
    let result = analyzer.analyze_files(&files)?;

    let formatter = OutputFormatter::new();
    Ok(formatter.format_type_errors(&result.type_errors, format))
}
```

This design provides a solid foundation for building a high-performance TypeScript analysis tool with rich output formatting capabilities.