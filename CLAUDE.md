# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**TA (TypeScript Analyzer)** is a high-performance AST analyzer written in Rust using OXC (Oxidation Compiler). It provides deep analysis capabilities for TypeScript codebases including type errors, symbol tracking, dependencies, and file watching.

## Architecture

This is a Rust workspace with three main modules:

1. **`/lib`** - Core library providing TypeScript analysis functionality
2. **`/cli`** - Command-line interface exposing library features
3. **`/ts`** - TypeScript type definitions for handler functions used with `ta watch`

### Key Design Points

- Uses OXC under the hood for fast AST parsing and analysis
- All CLI commands output data to STDOUT, status/progress to STDERR
- Three output formats: console (default with ANSI colors), JSON (`--json`), HTML (`--html`)
- JSON output includes both structured data and pre-formatted console/HTML representations
- HTML output includes metadata in `data-*` attributes on wrapper spans

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

## CLI Commands (ta)

All commands support `--json` and `--html` output flags.

### Type Analysis

```bash
ta source <filter>             # Analyze type errors in source files
                              # Excludes test files when analyzing whole repo
                              # Output includes scope, block context, and highlighted code
```

### Symbol Analysis

```bash
ta symbols <filter>            # List exported symbols with filtering by type
                              # Returns structured data + HTML/console representations
```

### Test Analysis

```bash
ta test                        # Detect and report on type tests
                              # Looks for describe()->it()/test() blocks
                              # Identifies type test patterns: type cases = [...]
```

### Dependency Analysis

```bash
ta file <filter>               # File-level dependencies (repo + external packages)
ta deps <filter>               # Symbol-level dependencies
                              # Scope: local, repo, module (monorepo), external
```

### File Watching

```bash
ta watch <handler> <handler>   # Watch for file changes and trigger handlers
                              # Syntax: --${Event} ${Executable}
```

**Watch Events:**
- `sourceFileChanged`, `sourceFileCreated`, `sourceFileRemoved`
- `symbolRenamed`, `symbolAdded`, `symbolRemoved`
- `moduleDepChanged`, `externalDepChanged`
- `testStatusChanged`, `newFailingTest`, `testFixed`, `newTestAdded`

**Handler Types:**
- Native executables (with exec permissions on POSIX)
- TypeScript files with Bun installed, exporting handler functions like:
  ```ts
  export const onSourceFileChanged: SourceFileChangedHandler = (evt) => { ... }
  ```

## Scope Information

The analyzer provides rich scope context for errors:

- `${file}::${symbol}` - Error inside a function
- `${file}::${class}:${method}` - Error inside a class method
- `${file}::root` - Error at module/file root level

## Output Formats

- **Console**: ANSI escape sequences for terminal color/formatting (least verbose)
- **HTML**: Span/class blocks with `data-*` metadata attributes
- **JSON**: Most verbose - includes all structured data plus console/HTML representations
