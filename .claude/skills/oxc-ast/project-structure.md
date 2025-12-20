# Project Structure

Professional Cargo workspace setup for OXC-based tools, separating core logic from WASM bindings.

## Recommended Structure

```text
oxc-analyzer/
├── Cargo.toml           # Workspace configuration
├── crates/
│   ├── core/            # Pure Rust analysis logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs   # Entry point
│   │       ├── visitor.rs # Custom AST visitors
│   │       └── models.rs  # Data structures
│   └── wasm/            # WASM Bindings
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs   # JS/TS interface
├── package.json         # WASM build scripts
└── ts-example/          # TypeScript usage examples
```

## Workspace Cargo.toml

```toml
[workspace]
members = ["crates/core", "crates/wasm"]
resolver = "2"

[workspace.dependencies]
oxc = { version = "0.30.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link Time Optimization
codegen-units = 1   # Better optimization, slower build
strip = true        # Remove debug symbols
```

## Core Crate (crates/core/Cargo.toml)

```toml
[package]
name = "oxc-analyzer-core"
version = "0.1.0"
edition = "2021"

[dependencies]
oxc = { workspace = true }
serde = { workspace = true }
rayon = "1.8"
dashmap = "5.5"
```

## Core Logic (crates/core/src/lib.rs)

```rust
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;

pub struct Analyzer<'a> {
    allocator: &'a Allocator,
    source: &'a str,
}

impl<'a> Analyzer<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self { allocator, source }
    }

    pub fn analyze(&self) -> Result<AnalysisResult, String> {
        let source_type = SourceType::default().with_typescript(true);
        let ret = Parser::new(self.allocator, self.source, source_type).parse();

        if !ret.errors.is_empty() {
            return Err(format!("Parse errors: {}", ret.errors.len()));
        }

        let semantic = SemanticBuilder::new().build(&ret.program).semantic;

        // Custom logic to extract interfaces, types, etc.
        let mut results = AnalysisResult::default();
        // ... (populate results using visitors)

        Ok(results)
    }
}

#[derive(Default, serde::Serialize)]
pub struct AnalysisResult {
    pub interfaces: Vec<String>,
    pub total_symbols: usize,
}
```

## WASM Crate (crates/wasm/Cargo.toml)

```toml
[package]
name = "oxc-analyzer-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
oxc-analyzer-core = { path = "../core" }
oxc = { workspace = true }
wasm-bindgen = "0.2"
serde = { workspace = true }
serde-wasm-bindgen = "0.6"
```

## WASM Bridge (crates/wasm/src/lib.rs)

```rust
use wasm_bindgen::prelude::*;
use oxc_analyzer_core::{Analyzer, AnalysisResult};
use oxc::allocator::Allocator;

#[wasm_bindgen]
pub fn analyze_code(source: &str) -> Result<JsValue, JsError> {
    let allocator = Allocator::default();
    let analyzer = Analyzer::new(&allocator, source);

    let result = analyzer.analyze()
        .map_err(|e| JsError::new(&e))?;

    Ok(serde_wasm_bindgen::to_value(&result)?)
}
```

## Package.json for WASM

```json
{
  "name": "oxc-analyzer-wasm",
  "version": "0.1.0",
  "scripts": {
    "build": "wasm-pack build crates/wasm --release --target nodejs --out-dir ../../pkg",
    "build:web": "wasm-pack build crates/wasm --release --target web --out-dir ../../pkg-web",
    "test": "wasm-pack test crates/wasm --node"
  },
  "devDependencies": {
    "wasm-pack": "^0.12.0"
  }
}
```

## CLI Crate (Optional)

Add a CLI for local testing:

```text
crates/
├── cli/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
```

**crates/cli/Cargo.toml:**

```toml
[package]
name = "oxc-analyzer-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "oxc-analyze"
path = "src/main.rs"

[dependencies]
oxc-analyzer-core = { path = "../core" }
clap = { version = "4.4", features = ["derive"] }
ignore = "0.4"
rayon = "1.8"
```

**crates/cli/src/main.rs:**

```rust
use clap::Parser;
use ignore::WalkBuilder;
use oxc_analyzer_core::{Analyzer, AnalysisResult};
use oxc::allocator::Allocator;
use rayon::prelude::*;

#[derive(Parser)]
#[command(name = "oxc-analyze")]
#[command(about = "Analyze TypeScript/JavaScript code")]
struct Cli {
    /// Directory to scan
    path: String,
}

fn main() {
    let cli = Cli::parse();

    let file_paths: Vec<_> = WalkBuilder::new(&cli.path)
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .and_then(|s| s.to_str())
                .map(|ext| matches!(ext, "ts" | "js" | "tsx" | "jsx"))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    println!("Analyzing {} files...", file_paths.len());

    let results: Vec<_> = file_paths
        .par_iter()
        .filter_map(|path| {
            let allocator = Allocator::default();
            let source = std::fs::read_to_string(path).ok()?;
            let analyzer = Analyzer::new(&allocator, &source);
            analyzer.analyze().ok()
        })
        .collect();

    let total_symbols: usize = results.iter().map(|r| r.total_symbols).sum();
    println!("Total symbols: {}", total_symbols);
}
```

## Pro-Tips for Production

### 1. Memory Management

Always reuse `Allocator` when possible **within a single thread**:

```rust
// Good: Reuse allocator for multiple small snippets
let allocator = Allocator::default();

for snippet in snippets {
    let ret = Parser::new(&allocator, snippet, source_type).parse();
    // Process...
}
```

### 2. Span Management

Convert byte offsets to UTF-16 for JavaScript/VS Code:

```rust
pub fn get_line_column_offset(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;

    for (i, c) in source.char_indices() {
        if i >= byte_offset { break; }

        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}
```

### 3. Error Handling with miette

Use `miette` for beautiful CLI errors:

```toml
[dependencies]
miette = { version = "7.0", features = ["fancy"] }
```

```rust
use miette::{Diagnostic, SourceSpan};

#[derive(Debug, Diagnostic)]
#[diagnostic(code(analyzer::parse_error))]
struct ParseError {
    #[source_code]
    src: String,

    #[label("error here")]
    span: SourceSpan,
}
```

## Build Scripts

**Makefile:**

```makefile
.PHONY: build test wasm cli

build:
	cargo build --release

test:
	cargo test --workspace

wasm:
	npm run build

cli:
	cargo build --release -p oxc-analyzer-cli

install-cli:
	cargo install --path crates/cli

clean:
	cargo clean
	rm -rf pkg pkg-web
```

## Testing Structure

```text
crates/core/
├── src/
└── tests/
    ├── fixtures/
    │   ├── input1.ts
    │   └── input2.ts
    └── integration_tests.rs
```

**integration_tests.rs:**

```rust
use oxc_analyzer_core::Analyzer;
use oxc::allocator::Allocator;

#[test]
fn test_interface_detection() {
    let source = include_str!("fixtures/input1.ts");
    let allocator = Allocator::default();
    let analyzer = Analyzer::new(&allocator, source);
    let result = analyzer.analyze().unwrap();

    assert_eq!(result.interfaces.len(), 2);
}
```

## Documentation Structure

```text
docs/
├── getting-started.md
├── api-reference.md
├── examples/
│   ├── basic-usage.md
│   └── advanced-patterns.md
└── architecture.md
```

## Release Checklist

- [ ] Update version in all `Cargo.toml` files
- [ ] Run `cargo test --workspace`
- [ ] Build WASM: `npm run build`
- [ ] Update `CHANGELOG.md`
- [ ] Tag release: `git tag v0.1.0`
- [ ] Publish to crates.io: `cargo publish -p oxc-analyzer-core`
- [ ] Publish to NPM: `cd pkg && npm publish`
