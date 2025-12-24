# Project Dependencies

## Structure

This is a Rust workspace with TypeScript handler types:

- `Cargo.toml` - Root workspace configuration
- `lib/Cargo.toml` - Core TypeScript analysis library
- `cli/Cargo.toml` - Command-line interface
- `ts/package.json` - TypeScript handler type definitions
- `scripts/package.json` - Build scripts and utilities

## Library Dependencies (`lib`)

### AST Parsing & Analysis

- [oxc_parser](https://github.com/oxc-project/oxc) **v0.30**

    High-performance TypeScript/JavaScript parser written in Rust. Uses memory arena allocation (bumpalo) for fast AST operations. Part of the Oxidation Compiler project.

- [oxc_semantic](https://docs.rs/oxc_semantic) **v0.30**

    Semantic analyzer providing symbol resolution and scope binding for JavaScript/TypeScript code.

- [oxc_allocator](https://docs.rs/oxc_allocator) **v0.30**

    Memory arena allocator for OXC AST nodes, enabling zero-cost AST drops.

- [oxc_span](https://docs.rs/oxc_span) **v0.30**

    Source position tracking for AST nodes and diagnostics.

- [oxc_ast](https://docs.rs/oxc_ast) **v0.30**

    Abstract syntax tree definitions for JavaScript and TypeScript.

- [oxc_diagnostics](https://docs.rs/oxc_diagnostics) **v0.30**

    Diagnostic and error reporting infrastructure for OXC.

### Serialization

- [serde](https://serde.rs/) **v1.0** (with `derive` feature)

    Industry-standard serialization framework for Rust. Provides `#[derive(Serialize, Deserialize)]` macros for automatic implementation.

- [serde_json](https://github.com/serde-rs/json) **v1.0**

    JSON serialization/deserialization using serde. Enables JSON output format for analysis results.

### Parallel Processing

- [rayon](https://github.com/rayon-rs/rayon) **v1.10**

    Data parallelism library using work-stealing thread pool. Used for parallel file processing with per-thread allocators.

### File System & Watching

- [glob](https://github.com/rust-lang-nursery/glob) **v0.3**

    Pattern matching for file paths supporting `**/*.ts` style globs.

- [notify](https://github.com/notify-rs/notify) **v7.0**

    Cross-platform filesystem notification library. Used as the foundation for watch mode.

- [notify-debouncer-full](https://github.com/notify-rs/notify) **v0.6.0**

    Debouncing wrapper for notify with 500ms delay to batch rapid file changes.

### Error Handling

- [thiserror](https://github.com/dtolnay/thiserror) **v2.0**

    Derive macro for `std::error::Error` trait. Provides `#[derive(Error)]` for custom error types with `#[from]` and `#[source]` attributes.

- [miette](https://github.com/zkat/miette) **v7** (with `derive` feature)

    Fancy diagnostic reporting library with beautiful error messages and code snippets.

### Logging

- [log](https://github.com/rust-lang/log) **v0.4**

    Lightweight logging facade providing macros like `info!`, `debug!`, `warn!`, and `error!`.

### Output Formatting

- [html-escape](https://github.com/magiclen/html-escape) **v0.2.13**

    HTML entity encoding for safe HTML output generation.

- [colored](https://github.com/mackwic/colored) **v2.1**

    Terminal color and styling library for ANSI escape codes in console output.

- [syntect](https://github.com/trishume/syntect) **v5.2** (with default syntaxes/themes, HTML, parsing)

    Syntax highlighting using Sublime Text definitions. Provides ~16,000 lines/second highlighting performance.

- [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) **v0.12**

    Efficient CommonMark/Markdown parser using pull-parsing approach. Supports GFM extensions including tables and task lists.

### CLI Utilities

- [clap](https://github.com/clap-rs/clap) **v4.5.53** (with `derive` feature)

    Command-line argument parser. Uses derive API with `#[derive(Parser)]` for declarative CLI definitions.

## CLI Dependencies (`cli`)

### Workspace Dependency

- **ta-lib** (path = `../lib`)

    Local workspace dependency providing core TypeScript analysis functionality.

### CLI Framework

- [clap](https://github.com/clap-rs/clap) **v4.5** (with `derive`, `env` features)

    Command-line argument parser with environment variable support.

### Error Handling & Logging

- [color-eyre](https://github.com/eyre-rs/color-eyre) **v0.6**

    Colorful error reports with panic hooks, backtraces, and span traces. Built on top of the eyre error handling library.

- [env_logger](https://github.com/rust-cli/env_logger) **v0.11**

    Simple logger configured via `RUST_LOG` environment variable. Logs to stderr by default with configurable levels.

- [log](https://github.com/rust-lang/log) **v0.4**

    Logging facade (same as lib dependency).

- [thiserror](https://github.com/dtolnay/thiserror) **v2.0**

    Error derive macros (same as lib dependency).

### File System & Output

- [glob](https://github.com/rust-lang-nursery/glob) **v0.3.3**

    File path pattern matching.

- [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) **v0.4**

    Fast recursive directory iterator respecting .gitignore, .ignore, and file type filters. Part of ripgrep.

- [serde_json](https://github.com/serde-rs/json) **v1.0.145**

    JSON output format support.

- [colored](https://github.com/mackwic/colored) **v2.0**

    Terminal color output.

### Terminal Detection

- [atty](https://github.com/softprops/atty) **v0.2** ⚠️ **DEPRECATED**

    TTY detection for determining if output is to a terminal. **Note:** This crate is deprecated - consider migrating to `std::io::IsTerminal` (Rust 1.70+) or `is-terminal` crate.

## TypeScript Handler Package (`ts`)

### Build Tools

- [tsdown](https://tsdown.dev/) **v0.18.1** (dev)

    Modern TypeScript/JavaScript bundler built on Rolldown (written in Rust). Generates ESM bundles and declaration files with blazing-fast performance.

- [typescript](https://www.typescriptlang.org/) **v5.9.3** (dev)

    TypeScript compiler and language services for type checking and declaration generation.

### Versioning

- [bumpp](https://github.com/antfu/bumpp) **v10.3.2** (dev)

    Interactive CLI for bumping package versions with git tag support.

## Build Scripts (`scripts`)

### Runtime

- [bun](https://bun.sh/) **v1.3.5**

    Fast JavaScript runtime and toolkit. Used for executing build scripts with native TypeScript support.

### Type Definitions

- [@types/bun](https://www.npmjs.com/package/@types/bun) **v1.3.5** (dev)

    TypeScript type definitions for Bun runtime APIs.

### Peer Dependencies

- [typescript](https://www.typescriptlang.org/) **v5** (peer)

    TypeScript compiler (peer dependency for scripts).

## Development & Testing Dependencies

### Testing (`lib/dev-dependencies`)

- [proptest](https://github.com/proptest-rs/proptest) **v1**

    Property-based testing framework inspired by Hypothesis. Generates arbitrary inputs and automatically shrinks failing test cases.

- [insta](https://insta.rs/) **v1**

    Snapshot testing library with VS Code integration. Automatically manages reference snapshots and provides beautiful diffs.

- [tempfile](https://github.com/Stebalien/tempfile) **v3**

    Secure cross-platform temporary file/directory creation with automatic cleanup on drop.

- [pretty_assertions](https://github.com/rust-pretty-assertions/rust-pretty-assertions) **v1**

    Better assertion failure messages with colorful diffs for easier debugging.

- [serial_test](https://github.com/palfrey/serial_test) **v3**

    Run tests serially using `#[serial]` attribute macro to avoid race conditions in filesystem tests.

### Benchmarking (`lib/dev-dependencies`)

- [criterion](https://github.com/bheisler/criterion.rs) **v0.5**

    Statistics-driven micro-benchmarking with automatic regression detection and chart generation using gnuplot.

### Integration Testing (`cli/dev-dependencies`)

- [assert_cmd](https://github.com/assert-rs/assert_cmd) **v2**

    Simplifies CLI integration testing with helpers for running binaries and asserting on exit codes, stdout, and stderr.

- [predicates](https://github.com/assert-rs/predicates-rs) **v3**

    Boolean-valued predicate functions for flexible assertions. Commonly used with assert_cmd for pattern matching in command outputs.

## Release Profile Optimizations

The workspace uses aggressive release optimizations:

- **opt-level = 3** - Maximum optimization
- **lto = true** - Link-time optimization across all crates
- **codegen-units = 1** - Single codegen unit for better optimization
- **strip = true** - Strip debug symbols from binary

These settings significantly improve OXC parser performance at the cost of longer compile times.
