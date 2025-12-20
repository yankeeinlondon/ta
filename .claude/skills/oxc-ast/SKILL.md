---
name: oxc-ast
description: Expert knowledge for building high-performance JavaScript/TypeScript AST analysis tools using OXC (Oxidation Compiler) in Rust, including parsing, semantic analysis, visitor patterns, parallel processing with Rayon, WASM compilation, custom linting, dead code detection, symbol renaming, and dependency extraction
last_updated: 2025-12-20T00:00:00Z
hash: f221f98d4928e07f
---

# OXC AST Analysis

Expert knowledge for building high-performance JavaScript/TypeScript AST tools using **OXC (Oxidation Compiler)** in Rust.

OXC is the fastest JavaScript/TypeScript parser and analyzer, powering tools like Oxlint. It uses arena allocation and semantic analysis to achieve 10-50x performance improvements over traditional JS-based tools.

## Core Principles

- **Use Semantic Analysis, not just AST walking** - OXC's `SemanticBuilder` pre-calculates scopes, symbols, and references for accurate analysis
- **Arena allocation is key** - OXC's `Allocator` provides zero-cost memory management; create one per thread/file
- **Leverage the flat node array** - OXC stores nodes in a flat arena (not a tree), making iteration faster than recursive traversal
- **Match on `AstKind`, not node types** - Use `semantic.nodes().iter()` with pattern matching on `AstKind` variants
- **Parallelize with Rayon** - OXC's per-thread `Allocator` design enables perfect parallelization across files
- **Return simple data from WASM** - Don't try to pass OXC's complex arena types to JS; serialize to JSON or return edit lists
- **Use `SymbolId` for identity, not names** - Symbol IDs prevent name collision bugs (e.g., two functions named `init()` in different files)
- **Compile to WASM for JS ecosystem** - Use `wasm-bindgen` to bring Rust performance to Node.js, VS Code extensions, and browsers
- **Respect UTF-8 vs UTF-16 spans** - OXC uses byte offsets (UTF-8); convert to character offsets for JS/VS Code APIs
- **Profile with `--release` always** - OXC's performance gains only appear in release builds with LTO enabled

## Quick Reference

### Basic Setup

```toml
[dependencies]
oxc = { version = "0.30.0", features = ["full"] }
```

```rust
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;

let allocator = Allocator::default();
let source_type = SourceType::default().with_typescript(true);
let ret = Parser::new(&allocator, source_code, source_type).parse();
let semantic = SemanticBuilder::new().build(&ret.program).semantic;
```

### Standard Analysis Pattern

```rust
use oxc::ast::AstKind;

for node in semantic.nodes().iter() {
    match node.kind() {
        AstKind::Function(func) => { /* process functions */ },
        AstKind::TSInterfaceDeclaration(decl) => { /* process interfaces */ },
        AstKind::CallExpression(call) => { /* process calls */ },
        _ => {}
    }
}
```

## Topics

### Core Architecture

- [OXC Components](./components.md) - Parser, AST, Semantic analysis, and how they work together
- [Visitor Pattern](./visitor-pattern.md) - Implementing custom AST visitors for type extraction and graph building
- [Semantic Model](./semantic-model.md) - Understanding symbols, scopes, and references for accurate analysis

### Practical Analysis

- [Finding Functions](./finding-functions.md) - Detecting all function types (declarations, expressions, arrows, methods)
- [Dead Code Detection](./dead-code.md) - Two-pass global analysis for unused functions across files
- [Symbol Renaming](./renaming.md) - Precise refactoring using the Symbol Table and reference graph
- [Custom Lint Rules](./lint-rules.md) - Building context-aware linting with semantic knowledge

### Performance & Scale

- [Parallel Processing](./parallel-processing.md) - Using Rayon for multi-threaded file scanning
- [JSON Reporting](./json-reporting.md) - Generating structured reports for CI/CD integration
- [Project Structure](./project-structure.md) - Professional Cargo workspace setup with core/WASM separation

### WASM Integration

- [WASM Compilation](./wasm-compilation.md) - Compiling to WebAssembly for Node.js and browser use
- [WASM CI/CD](./wasm-cicd.md) - GitHub Actions pipeline for automated NPM publishing

## Common Patterns

### Extract Dependencies

```rust
use oxc::ast::ast::{ModuleDeclaration, Statement};

for statement in &ret.program.body {
    if let Statement::ImportDeclaration(import_decl) = statement {
        println!("Dependency: {}", import_decl.source.value);
        if let Some(specifiers) = &import_decl.specifiers {
            for spec in specifiers {
                println!("  Imports: {:?}", spec.local.name);
            }
        }
    }
}
```

### Extract Type Annotations

```rust
use oxc::ast::ast::TSType;

if let AstKind::VariableDeclarator(decl) = node.kind() {
    if let Some(ident) = decl.id.get_identifier() {
        if let Some(type_annotation) = &decl.id.type_annotation {
            match &type_annotation.type_annotation {
                TSType::TSNumberKeyword(_) => println!("{}: number", ident),
                TSType::TSStringKeyword(_) => println!("{}: string", ident),
                TSType::TSTypeReference(r) => println!("{}: {:?}", ident, r.type_name),
                _ => {}
            }
        }
    }
}
```

### Find Unused Symbols (Per File)

```rust
let symbols = semantic.symbols();
for symbol_id in symbols.symbol_ids() {
    let name = symbols.get_name(symbol_id);
    let refs = symbols.get_resolved_references(symbol_id);

    if refs.is_empty() {
        println!("Unused symbol: {}", name);
    }
}
```

## Resources

- [OXC GitHub](https://github.com/oxc-project/oxc)
- [OXC Documentation](https://oxc-project.github.io/)
- [Rayon Parallel Iterator Docs](https://docs.rs/rayon/latest/rayon/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
