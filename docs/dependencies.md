# Project Dependencies

## Structure

This is a Rust workspace with an additional TypeScript package:

- `Cargo.toml` - Workspace configuration
- `lib/Cargo.toml` - Core library (ta-lib)
- `cli/Cargo.toml` - Command-line interface (ta)
- `ts/package.json` - TypeScript handler types package (ta-handler)

## Rust Production Dependencies

### [OXC (Oxidation Compiler)](https://github.com/oxc-project/oxc)

**Versions:** 0.30 (multiple crates)

High-performance JavaScript and TypeScript tooling suite written in Rust. This project uses multiple OXC crates:

- **oxc_parser** - Fast and conformant JavaScript/TypeScript parser supporting JSX, TSX, and Stage 3 Decorators
- **oxc_semantic** - Semantic analysis providing symbol resolution and scope binding
- **oxc_allocator** - Memory arena allocator for fast AST allocation and deallocation
- **oxc_span** - Source code span and position tracking
- **oxc_ast** - Abstract Syntax Tree definitions
- **oxc_diagnostics** - Diagnostic reporting infrastructure

### [clap](https://github.com/clap-rs/clap)

**Version:** 4.5

Full-featured, fast command-line argument parser for Rust. Provides both derive-based and builder-based APIs for creating CLI interfaces with comprehensive features including help generation, version display, subcommands, and argument validation. Used with the `derive` and `env` features.

### [color-eyre](https://github.com/eyre-rs/eyre)

**Version:** 0.6

Custom error report handler providing colorful, human-oriented error reports. Part of the eyre ecosystem, it enhances panic messages and error reporting with backtraces, span traces, and consistent formatting.

### [thiserror](https://github.com/dtolnay/thiserror)

**Version:** 2.0

Convenient derive macro for implementing `std::error::Error` trait. Eliminates boilerplate in error type definitions using attributes like `#[error("...")]` for Display messages and `#[from]` for automatic error conversion.

### [miette](https://github.com/zkat/miette)

**Version:** 7

Fancy diagnostic reporting library extending `std::error::Error` with pretty, detailed diagnostic printing. Provides protocols for custom error reports with rich formatting and context. Used with the `derive` feature.

### [rayon](https://github.com/rayon-rs/rayon)

**Version:** 1.10

Data parallelism library for Rust. Provides lightweight parallel iterators that convert sequential computations into parallel ones with minimal code changes (e.g., `iter()` → `par_iter()`). Guarantees data-race freedom.

### [notify](https://github.com/notify-rs/notify)

**Version:** 7.0

Cross-platform filesystem notification library for Rust. Monitors file system events across different operating systems. Used by notable projects including cargo watch, rust-analyzer, and watchexec.

### [notify-debouncer-full](https://crates.io/crates/notify-debouncer-full)

**Version:** 0.6.0

Debounced file system event handler built on top of notify. Prevents rapid-fire events by grouping related filesystem changes together.

### [glob](https://crates.io/crates/glob)

**Version:** 0.3

Pattern matching library for filesystem paths supporting wildcards like `*` and `**`.

### [serde](https://github.com/serde-rs/serde)

**Version:** 1.0

Framework for serializing and deserializing Rust data structures efficiently and generically. Used with the `derive` feature for automatic serialization implementation.

### [serde_json](https://github.com/serde-rs/json)

**Version:** 1.0

JSON serialization/deserialization support for serde. Provides fast and correct JSON parsing and generation.

### [env_logger](https://github.com/rust-cli/env_logger)

**Version:** 0.11

Simple logger configured via environment variables. Commonly used with the `log` crate for flexible logging configuration.

### [log](https://github.com/rust-lang/log)

**Version:** 0.4

Lightweight logging facade providing a single logging API that abstracts over the actual logging implementation.

### [html-escape](https://crates.io/crates/html-escape)

**Version:** 0.2.13

Fast and correct HTML entity encoding and decoding library for preventing XSS vulnerabilities when generating HTML output.

## Rust Development Dependencies

### [assert_cmd](https://github.com/assert-rs/assert_cmd)

**Version:** 2

Testing framework for CLI applications. Simplifies testing command-line programs by providing assertions for exit codes, stdout, stderr, and more.

### [predicates](https://github.com/assert-rs/predicates-rs)

**Version:** 3

Boolean-valued functions for making assertions about values. Often used with assert_cmd for flexible test assertions.

### [proptest](https://github.com/proptest-rs/proptest)

**Version:** 1

Property-based testing framework for Rust inspired by QuickCheck. Generates random test cases to find edge cases automatically.

### [insta](https://github.com/mitsuhiko/insta)

**Version:** 1

Snapshot testing library that stores test outputs and detects changes across test runs. Excellent for testing complex outputs like AST structures or formatted text.

### [tempfile](https://github.com/Stebalien/tempfile)

**Version:** 3

Library for creating temporary files and directories that are automatically cleaned up when dropped.

### [pretty_assertions](https://github.com/rust-pretty-assertions/rust-pretty-assertions)

**Version:** 1

Drop-in replacement for `assert_eq!` and `assert_ne!` that provides colorful diffs when assertions fail, making test failures easier to diagnose.

## TypeScript Dependencies

### [tsdown](https://github.com/rolldown/tsdown)

**Version:** 0.18.1

The elegant bundler for TypeScript libraries powered by Rolldown. Provides blazing-fast builds and declaration file generation using Oxc and Rolldown. Supports Rollup, Rolldown, and unplugin plugins.

### [typescript](https://github.com/microsoft/TypeScript)

**Version:** 5.9.3

TypeScript language compiler and tooling. Provides type checking and compilation from TypeScript to JavaScript.

### [bumpp](https://github.com/antfu-collective/bumpp)

**Version:** 10.3.2

Interactive CLI tool for bumping version numbers. Supports conventional commits, monorepo workflows, and automatic changelog generation. Provides options for commit, tag, and push operations.
