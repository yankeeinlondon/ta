# AGENTS.md - Development Guidelines for TA (TypeScript Analyzer)

## Build & Test Commands

- **Build**: `cargo build` (debug) or `cargo build --release`
- **Test all**: `cargo test`
- **Test single**: `cargo test <test_name>` or `cargo test -- --exact <test_name>`
- **Test package**: `cargo test -p lib` (library) or `cargo test -p cli` (CLI)
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt`
- **Run CLI**: `cargo run -p cli -- <args>`

## TypeScript Commands

- **Build TS types**: `cd ts/ && pnpm build`
- **Test TS**: `cd ts/ && pnpm test` (no-op currently)

## Code Style Guidelines

### Rust

- **Error handling**: Use `Result<T, E>` with `?` operator; prefer `thiserror` for custom errors
- **Imports**: Group std, external crates, then local modules; use explicit imports
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants
- **Documentation**: Doc comments with examples for public APIs using `//!` for modules, `///` for items
- **Testing**: Unit tests in `#[cfg(test)] mod tests` blocks; integration tests in `tests/` directory
- **Async**: Use `#[instrument]` from tracing for async functions
- **Ownership**: Follow ownership principles; minimize clones; use references where possible

### TypeScript

- **Package manager**: pnpm (specified in package.json)
- **Module format**: ESM with type definitions
- **Build tool**: tsdown for bundling and DTS generation

### General

- **Formatting**: Follow rustfmt for Rust, Prettier for TypeScript
- **Linting**: clippy for Rust, eslint for TypeScript (if added)
- **Commits**: Clear, imperative messages; reference issues when applicable
- **Documentation**: Keep docs in `docs/` directory; update README for user-facing changes
