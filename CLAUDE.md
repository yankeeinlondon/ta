# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**TA (TypeScript Analyzer)** is a high-performance AST analyzer written in Rust using OXC (Oxidation Compiler). It provides deep analysis capabilities for TypeScript codebases including type errors, symbol tracking, dependencies, and file watching.

**Current Status:** Core functionality implemented (v0.1.0). See README roadmap for planned features.

## Architecture

This is a Rust workspace with three main modules:

1. **`/lib`** - Core library providing TypeScript analysis functionality
   - OXC 0.30 for parsing and semantic analysis
   - Rayon for parallel file processing
   - Custom visitor pattern implementations
   - Error types use thiserror

2. **`/cli`** - Command-line interface exposing library features
   - Built with clap 4.5 (derive API)
   - Error handling with color-eyre
   - Three output formats via OutputFormatter

3. **`/ts`** - TypeScript type definitions for handler functions used with `ta watch`
   - Discriminated union types for watch events
   - Designed for Bun runtime (handler execution not yet implemented)

### Key Design Points

- **OXC Integration:** Uses semantic analysis, not just AST walking
- **Output Separation:** Data to STDOUT, status/progress to STDERR
- **Three Formats:** Console (ANSI colors), JSON (serde), HTML (with data-* attributes)
- **Parallel Processing:** Rayon used with per-file allocators
- **Error Propagation:** Library uses thiserror, CLI wraps with color-eyre

## Commands

### Rust

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build

# Test
cargo test                     # Run all tests
cargo test -p lib              # Test library only
cargo test -p cli              # Test CLI only

# Run CLI
cargo run -p cli               # Run CLI tool
```

### TypeScript Handler Package

```bash
cd ts/
pnpm build                     # Build TypeScript handler types
pnpm test                      # No tests defined
```

## Known Issues & TODOs

### Must Fix Before 1.0
1. **TypeError.file field** - Always "unknown", not propagated from analyzer (type_error_visitor.rs:64)
2. **TypeError.id field** - Hardcoded to "error", should parse from OxcDiagnostic (type_error_visitor.rs:62)
3. **--include-tests flag** - Declared but unused (source.rs:19)
4. **Glob pattern validation** - No check for `..` path traversal (source.rs:38)
5. **External handler execution** - Watch mode only logs to console (watch.rs)

### Suggested Improvements
1. **Clippy warnings** - 2 needless lifetime warnings in dependencies.rs and tests.rs
2. **color-eyre usage** - Add `.with_help()` for user-actionable error messages
3. **Test coverage** - 26 tests vs 100+ target (missing: analyzer.rs, watcher.rs)
4. **JSON output format** - Should include console/HTML representations per README spec

### Performance Considerations
- Always use `--release` builds for OXC performance gains
- LTO enabled in release profile (lto = true, codegen-units = 1)
- Rayon parallelism only helps with multiple files (overhead for single files)

## CLI Commands (ta)

All commands support `--format <type>` where type is `console` (default), `json`, or `html`.

### Type Analysis

```bash
ta source [pattern]            # Analyze type errors in source files
  --filter <text>              # Filter by message or scope
  --max-errors <n>             # Limit results (default: 100)
  --include-tests              # Include test files (NOT YET IMPLEMENTED)
```

**Known Issues:**
- Error IDs are hardcoded to "error" (should parse from OXC diagnostics)
- File path always shows "unknown" in TypeError structs (not propagated)

### Symbol Analysis

```bash
ta symbols [pattern]           # Extract exported symbols
  --exported-only              # Only show exports (default shows all)
```

**Detects:** Functions, Classes, Interfaces, Types, Enums, Variables
**Includes:** Parameters for functions, properties for classes

### Test Analysis

```bash
ta test [pattern]              # Detect type tests in test files
```

Looks for `describe() → it()/test()` blocks with `type cases = [...]` patterns.

### Dependency Analysis

```bash
ta file [pattern]              # File-level dependencies (import/export statements)
ta deps [pattern]              # Symbol-level deps (NOT FULLY IMPLEMENTED)
```

File analysis detects:
- `import` declarations
- `export ... from` statements
- Re-exports (`export *`)

### File Watching

```bash
ta watch [paths...]            # Watch filesystem for changes
```

**Current Implementation:**
- Monitors .ts/.tsx files with 500ms debouncing
- Detects: SymbolAdded, SymbolRemoved, TestStatusChanged, NewFailingTest, TestFixed, NewTestAdded
- Logs events to console only

**NOT YET IMPLEMENTED:**
- External handler execution (`--${Event} ${Executable}` syntax)
- TypeScript handler support with Bun
- Events: SourceFileChanged, SourceFileCreated, SourceFileRemoved, SymbolRenamed, ModuleDepChanged, ExternalDepChanged

## Implementation Details

### Scope Tracking (TypeErrorVisitor)

The analyzer tracks scope context for errors using a stack pattern:

- `${symbol}` - Error inside a function (e.g., "foo")
- `${class}::${method}` - Error inside a class method (e.g., "MyClass::doSomething")
- `global` - Error at module/file root level

**Implementation:** Visitor pushes/pops scope names as it traverses the AST.

### Output Formats

- **Console**: ANSI escape codes (RED, BLUE, CYAN, etc.) for syntax highlighting
- **HTML**: Semantic elements with CSS classes (`error-block`, `keyword`, `type`)
- **JSON**: serde_json serialization of data structures

**Note:** JSON does NOT currently include console/HTML representations (planned feature).

### Visitor Pattern

All analysis uses OXC's visitor pattern:
- `SymbolVisitor` - Tracks `is_exporting` flag to filter symbols
- `TypeErrorVisitor` - Manages `current_scope` stack and `processed_errors` HashSet
- `DependencyVisitor` - Extracts import/export source strings
- `TestVisitor` - Looks for describe/it/test AST patterns

### Parallel Processing

Analyzer uses Rayon's `par_iter()` when `AnalysisOptions.parallel = true`:
- Each thread gets its own OXC `Allocator`
- Collects results into vectors then extends main result
- Disabled for single-file analysis

### File Watching

Uses `notify-debouncer-full` with 500ms delay:
- Recursive monitoring via `RecursiveMode::Recursive`
- Filters for `.ts` and `.tsx` extensions
- Computes diff between previous/current `AnalysisResult`
- Trait-based handler system (`WatchHandler` trait)
