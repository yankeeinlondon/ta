---
name: oxc-ast
description: Comprehensive guide to using OXC for AST-based static analysis, type extraction, and code transformation
created: 2025-12-20
last_updated: 2025-12-20T00:00:00Z
hash: f99d94edf8792e2d
tags:
  - rust
  - typescript
  - ast
  - oxc
  - static-analysis
  - wasm
---

# OXC AST: Comprehensive Guide

This guide explores using the **Oxidation Compiler (OXC)** for AST-based static analysis in Rust. OXC differs from other JavaScript tooling because it prioritizes performance through **memory arenas** and a **semantic analysis pass** (`oxc_semantic`) that pre-calculates scopes, symbols, and references.

## Table of Contents

1. [Core Concepts & Architecture](#core-concepts--architecture)
2. [Project Structure & Setup](#project-structure--setup)
3. [Extracting Type Information](#extracting-type-information)
4. [Finding Functions](#finding-functions)
5. [Visitor Pattern for Complex Extraction](#visitor-pattern-for-complex-extraction)
6. [Parallel Processing with Rayon](#parallel-processing-with-rayon)
7. [Dead Code Detection](#dead-code-detection)
8. [Custom Lint Rules](#custom-lint-rules)
9. [Structured JSON Reports](#structured-json-reports)
10. [WASM Integration](#wasm-integration)
11. [CI/CD Pipeline](#cicd-pipeline)
12. [TypeScript Integration](#typescript-integration)
13. [Production Best Practices](#production-best-practices)

## Core Concepts & Architecture

OXC provides a multi-layered architecture for working with JavaScript and TypeScript code:

- **`oxc_parser`**: Generates the AST. It allocates nodes into a memory arena (`oxc_allocator`), making allocation/deallocation extremely fast.
- **`oxc_ast`**: The AST definitions. It supports TypeScript, JSX, and latest ECMAScript. It is similar to ESTree but strictly typed (e.g., `BindingIdentifier` vs `IdentifierReference`).
- **`oxc_semantic`**: The "brain." It consumes the AST and produces a `Semantic` struct containing:
  - **Symbol Table:** All variables/functions declared.
  - **Scope Tree:** Nested scopes (block, function, module).
  - **Reference Graph:** Links usages (`x`) to declarations (`let x`).

### When to Use What

1. **Use `oxc_parser` + `Visitor`**: When you need to extract raw structure, dependencies, or generate documentation/graphs. It is fast and purely syntactic.
2. **Use `oxc_semantic`**: When you need to know *identity* (e.g., "Is this variable `x` the same as that variable `x` defined 50 lines up?"). Use this for linting, refactoring tools, or deep validation.

## Project Structure & Setup

### Directory Structure

```text
oxc-analyzer/
├── Cargo.toml           # Workspace configuration
├── crates/
│   ├── core/            # The heavy-lifting Rust logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs   # Entry point for analysis logic
│   │       ├── visitor.rs # Custom AST visitors
│   │       └── models.rs  # Shared data structures
│   └── wasm/            # WASM Bindings
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs   # JS/TS interface
├── package.json         # Scripts for wasm-pack and distribution
└── ts-example/          # TypeScript implementation example
```

### Basic Dependencies

```toml
[dependencies]
oxc = { version = "0.30.0", features = ["full"] }
```

### Core Library Structure

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

        let mut results = AnalysisResult::default();
        // populate results using visitors

        Ok(results)
    }
}

#[derive(Default, serde::Serialize)]
pub struct AnalysisResult {
    pub interfaces: Vec<String>,
    pub total_symbols: usize,
}
```

### Production Cargo.toml Configuration

```toml
[workspace]
members = ["crates/core", "crates/wasm"]
resolver = "2"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link Time Optimization
codegen-units = 1   # Better optimization, slower build
strip = true        # Remove symbols for smaller WASM
```

## Extracting Type Information

OXC can extract explicit type annotations (what is written in the code). Type inference is currently experimental.

### Basic Type Extraction

```rust
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use oxc::ast::AstKind;

fn analyze_types_and_scopes(source_code: &str) {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);

    // 1. Parse
    let ret = Parser::new(&allocator, source_code, source_type).parse();

    // 2. Build Semantic Model (Symbol Table + Scopes)
    let semantic = SemanticBuilder::new()
        .build(&ret.program)
        .semantic;

    // 3. Walk the Semantic Nodes
    for node in semantic.nodes() {
        if let AstKind::VariableDeclarator(decl) = node.kind() {
            if let Some(ident) = decl.id.get_identifier() {
                print!("Variable declared: {}", ident);

                // 4. Extract Explicit Type Annotation
                if let Some(type_annotation) = &decl.id.type_annotation {
                    match &type_annotation.type_annotation {
                        oxc::ast::ast::TSType::TSNumberKeyword(_) => println!(" [Type: number]"),
                        oxc::ast::ast::TSType::TSStringKeyword(_) => println!(" [Type: string]"),
                        oxc::ast::ast::TSType::TSTypeReference(r) => {
                            println!(" [Type: Reference to {:?}]", r.type_name);
                        }
                        _ => println!(" [Type: Complex/Other]"),
                    }
                } else {
                    println!(" [No explicit type]");
                }
            }
        }
    }
}
```

### Dependency Extraction

```rust
use oxc::allocator::Allocator;
use oxc::parser::{Parser, ParserReturn};
use oxc::span::SourceType;
use oxc::ast::ast::{ModuleDeclaration, Statement};

fn extract_dependencies(source_code: &str, filename: &str) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(filename).unwrap();

    let ret: ParserReturn = Parser::new(&allocator, source_code, source_type).parse();

    if !ret.errors.is_empty() {
        eprintln!("Parse error: {:?}", ret.errors);
        return;
    }

    // Iterate top-level body for imports/exports
    for statement in &ret.program.body {
        if let Statement::ImportDeclaration(import_decl) = statement {
            println!("Dependency found: {}", import_decl.source.value);

            // Extract named specifiers
            if let Some(specifiers) = &import_decl.specifiers {
                for spec in specifiers {
                    println!("  - Imports symbol: {:?}", spec.local.name);
                }
            }
        }
    }
}
```

### Type Errors & Diagnostics

```rust
// Accessing errors from the Semantic pass
let semantic_ret = SemanticBuilder::new().build(&ret.program);

for error in semantic_ret.errors {
    let error_with_source = error.with_source_code(source_code.to_string());
    println!("{:?}", error_with_source);
}
```

## Finding Functions

To find all functions in a codebase using OXC, you need to account for three different ways functions appear in JavaScript/TypeScript: **Function Declarations**, **Function Expressions**, and **Arrow Functions**.

### Multi-Pattern Function Visitor

```rust
use oxc::allocator::Allocator;
use oxc::ast::ast::{Function, FunctionKind};
use oxc::ast::visit::{walk, Visit};
use oxc::parser::Parser;
use oxc::span::SourceType;

struct FunctionFinder {
    functions: Vec<FunctionMetadata>,
}

#[derive(Debug)]
struct FunctionMetadata {
    name: String,
    is_async: bool,
    is_generator: bool,
    params_count: usize,
}

impl<'a> Visit<'a> for FunctionFinder {
    // Handle standard declarations and expressions
    fn visit_function(&mut self, func: &Function<'a>) {
        let name = func
            .id
            .as_ref()
            .map(|id| id.name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());

        self.functions.push(FunctionMetadata {
            name,
            is_async: func.r#async,
            is_generator: func.generator,
            params_count: func.params.items.len(),
        });

        walk::walk_function(self, func);
    }

    // Handle Arrow Functions
    fn visit_arrow_function_expression(&mut self, func: &oxc::ast::ast::ArrowFunctionExpression<'a>) {
        self.functions.push(FunctionMetadata {
            name: "arrow".to_string(),
            is_async: func.r#async,
            is_generator: false,
            params_count: func.params.items.len(),
        });

        walk::walk_arrow_function_expression(self, func);
    }
}

fn main() {
    let source_code = r#"
        async function fetchData(url: string) { return await fetch(url); }
        const process = (data: any) => { console.log(data); };
        function* gen() { yield 1; }
    "#;

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = Parser::new(&allocator, source_code, source_type).parse();

    let mut finder = FunctionFinder { functions: vec![] };
    finder.visit_program(&ret.program);

    for func in finder.functions {
        println!("{:?}", func);
    }
}
```

### Key Differences in AST Nodes

| Function Type | AST Node | Common Properties |
| --- | --- | --- |
| **Declaration** | `Function` | `id` is present, `kind` is `FunctionDeclaration` |
| **Expression** | `Function` | `id` may be null, `kind` is `FunctionExpression` |
| **Arrow** | `ArrowFunctionExpression` | No `id`, `expression` boolean for shorthand |
| **Class Method** | `MethodDefinition` | Contains a `Function` inside its `value` property |

### Finding Methods

```rust
fn visit_method_definition(&mut self, it: &oxc::ast::ast::MethodDefinition<'a>) {
    let name = it.key.static_name().unwrap_or("dynamic_method".into());
    println!("Found method: {}", name);

    walk::walk_method_definition(self, it);
}
```

### Refining with Semantic Analysis

Using `oxc_semantic`, you can find where a function is **called** versus where it is **defined** by looking at the Symbol Table:

```rust
let symbols = semantic.symbols();
for symbol_id in symbols.symbol_ids() {
    let name = symbols.get_name(symbol_id);
    let resolved_refs = symbols.get_resolved_references(symbol_id);

    if resolved_refs.is_empty() {
        println!("Symbol {} is never used in this file.", name);
    }
}
```

## Visitor Pattern for Complex Extraction

The Visitor Pattern is ideal for building **Type Dependency Graphs** and extracting complex relationships.

### Interface Inheritance Graph

Add `petgraph` to your dependencies:

```toml
[dependencies]
oxc = { version = "0.30.0", features = ["full"] }
petgraph = "0.6"
```

```rust
use oxc::allocator::Allocator;
use oxc::ast::ast::{TSInterfaceDeclaration, TSType};
use oxc::ast::visit::{walk, Visit};
use oxc::parser::Parser;
use oxc::span::SourceType;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::dot::{Dot, Config};
use std::collections::HashMap;

struct TypeGraph {
    graph: DiGraph<String, ()>,
    indices: HashMap<String, NodeIndex>,
}

impl TypeGraph {
    fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            indices: HashMap::new(),
        }
    }

    fn add_node(&mut self, name: &str) -> NodeIndex {
        if let Some(&idx) = self.indices.get(name) {
            return idx;
        }
        let idx = self.graph.add_node(name.to_string());
        self.indices.insert(name.to_string(), idx);
        idx
    }

    fn add_dependency(&mut self, from: &str, to: &str) {
        let from_idx = self.add_node(from);
        let to_idx = self.add_node(to);
        self.graph.add_edge(from_idx, to_idx, ());
    }
}

struct InterfaceVisitor<'a> {
    graph: TypeGraph,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> InterfaceVisitor<'a> {
    fn new() -> Self {
        Self {
            graph: TypeGraph::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a> Visit<'a> for InterfaceVisitor<'a> {
    fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
        let interface_name = it.id.name.as_str();
        self.graph.add_node(interface_name);

        if let Some(extends) = &it.extends {
            for heritage in extends {
                if let oxc::ast::ast::Expression::IdentifierReference(ident) = &heritage.expression {
                    let parent_name = ident.name.as_str();
                    println!("Found relationship: {} extends {}", interface_name, parent_name);
                    self.graph.add_dependency(interface_name, parent_name);
                }
            }
        }

        walk::walk_ts_interface_declaration(self, it);
    }
}

fn main() {
    let source_code = r#"
        interface Entity {
            id: string;
        }
        interface User extends Entity {
            username: string;
        }
        interface Admin extends User {
            permissions: string[];
        }
    "#;

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = Parser::new(&allocator, source_code, source_type).parse();

    let mut visitor = InterfaceVisitor::new();
    visitor.visit_program(&ret.program);

    println!("{:?}", Dot::with_config(&visitor.graph.graph, &[Config::EdgeNoLabel]));
}
```

### Extracting Complex Type Info

To extract the **shape** of an interface (keys and value types):

```rust
fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
    println!("Interface: {}", it.id.name);

    for signature in &it.body.body {
        if let oxc::ast::ast::TSSignature::TSPropertySignature(prop) = signature {
            let key = match &prop.key {
                oxc::ast::ast::PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                _ => "computed_key",
            };

            let type_name = if let Some(ann) = &prop.type_annotation {
                match &ann.type_annotation {
                    TSType::TSStringKeyword(_) => "string",
                    TSType::TSNumberKeyword(_) => "number",
                    TSType::TSTypeReference(r) => "some_reference",
                    _ => "complex",
                }
            } else {
                "any"
            };

            println!("  - Field: {} -> {}", key, type_name);
        }
    }

    walk::walk_ts_interface_declaration(self, it);
}
```

## Parallel Processing with Rayon

For large codebases, parallel processing is essential. Use `rayon` to distribute file processing across CPU cores.

### Dependencies

```toml
[dependencies]
oxc = { version = "0.30.0", features = ["full"] }
rayon = "1.8"
ignore = "0.4"  # Fast directory traversal that respects .gitignore
```

### Parallel Function Scanner

```rust
use ignore::WalkBuilder;
use oxc::allocator::Allocator;
use oxc::ast::AstKind;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct FuncInfo {
    file: String,
    name: String,
    line: usize,
}

fn main() {
    let target_dir = ".";
    let total_files = AtomicUsize::new(0);

    // Collect all valid files
    let file_paths: Vec<PathBuf> = WalkBuilder::new(target_dir)
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|e| {
            let path = e.path();
            path.is_file() &&
            matches!(path.extension().and_then(|s| s.to_str()), Some("ts" | "js" | "tsx" | "jsx"))
        })
        .map(|e| e.into_path())
        .collect();

    println!("Scanning {} files...", file_paths.len());

    // Process in Parallel
    let all_functions: Vec<FuncInfo> = file_paths
        .par_iter()
        .filter_map(|path| {
            total_files.fetch_add(1, Ordering::SeqCst);

            let allocator = Allocator::default();
            let source_code = std::fs::read_to_string(path).ok()?;
            let source_type = SourceType::from_path(path).unwrap_or_default();

            let ret = Parser::new(&allocator, &source_code, source_type).parse();
            let semantic = SemanticBuilder::new().build(&ret.program).semantic;

            let mut local_funcs = Vec::new();

            for node in semantic.nodes().iter() {
                match node.kind() {
                    AstKind::Function(func) => {
                        let name = func.id.as_ref()
                            .map(|id| id.name.to_string())
                            .unwrap_or_else(|| "anonymous".to_string());

                        let line = source_code[..func.span.start as usize].lines().count();

                        local_funcs.push(FuncInfo {
                            file: path.to_string_lossy().into_owned(),
                            name,
                            line,
                        });
                    }
                    _ => {}
                }
            }
            Some(local_funcs)
        })
        .flatten()
        .collect();

    for func in &all_functions {
        println!("{}:{} -> function {}", func.file, func.line, func.name);
    }

    println!("\nFinished. Found {} functions in {} files.",
        all_functions.len(), total_files.load(Ordering::SeqCst));
}
```

### Why This is Fast

- **Zero-Cost Abstractions**: Rayon uses "work-stealing" to ensure efficient load balancing
- **Arena Allocation**: Each thread maintains its own `Allocator`, avoiding lock contention
- **Sequential I/O vs. Parallel Analysis**: CPU-intensive parsing is distributed across cores

### Performance Note

In large codebases, you should:

1. Initialize the `Allocator` inside the thread/worker
2. Parse and analyze the file entirely within that thread
3. Return only the "plain" data back to your main thread

OXC's `Allocator` and `Program` are not `Send` due to arena pointers, so they must stay within their thread.

## Dead Code Detection

Detecting unused code requires a **two-pass analysis** across the entire codebase.

### The Two-Pass Strategy

1. **Pass 1 (Parallel)**: Scan every file to collect all **Declarations** (Symbols) and all **Usages** (References)
2. **Pass 2 (Global)**: Reconcile the two. If a symbol is marked as "Exported" but no other file imports or calls it, it is potentially dead

### Global Dead Code Detector

```rust
use dashmap::DashSet;
use oxc::allocator::Allocator;
use oxc::ast::AstKind;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use rayon::prelude::*;
use std::sync::Arc;

struct DeadCodeDetector {
    declared_functions: DashSet<String>,
    used_functions: DashSet<String>,
}

fn main() {
    let detector = Arc::new(DeadCodeDetector {
        declared_functions: DashSet::new(),
        used_functions: DashSet::new(),
    });

    let file_paths = vec![/* paths */];

    // PASS 1: Collection
    file_paths.par_iter().for_each(|path| {
        let allocator = Allocator::default();
        let source = std::fs::read_to_string(path).unwrap();
        let ret = Parser::new(&allocator, &source, SourceType::from_path(path).unwrap()).parse();
        let semantic = SemanticBuilder::new().build(&ret.program).semantic;

        for node in semantic.nodes().iter() {
            match node.kind() {
                AstKind::Function(func) => {
                    if let Some(id) = &func.id {
                        detector.declared_functions.insert(id.name.to_string());
                    }
                }
                AstKind::CallExpression(call) => {
                    if let oxc::ast::ast::Expression::IdentifierReference(ident) = &call.callee {
                        detector.used_functions.insert(ident.name.to_string());
                    }
                }
                _ => {}
            }
        }
    });

    // PASS 2: Reconciliation
    println!("--- Potentially Unused Functions ---");
    for func in detector.declared_functions.iter() {
        if !detector.used_functions.contains(func.key()) {
            println!("Unused: {}", func.key());
        }
    }
}
```

### Handling False Positives

The naive approach doesn't account for **Module Exports**. For production:

1. **Identify Exports**: Check if the function is wrapped in an `ExportNamedDeclaration`
2. **Trace Imports**: Use `oxc_resolver` to follow imports
3. **Entry Points**: Define entry points (like `index.ts`). Any function not reachable via the call graph is "dead"

### Refining with Symbol ID

To avoid name collisions, use OXC's unique `SymbolId`:

```rust
let symbols = semantic.symbols();
for symbol_id in symbols.symbol_ids() {
    let name = symbols.get_name(symbol_id);
    let resolved_refs = symbols.get_resolved_references(symbol_id);

    if resolved_refs.is_empty() {
        println!("Symbol {} is never used in this file.", name);
    }
}
```

## Custom Lint Rules

### Implementing a Lint Rule

Example: "Interfaces must not be prefixed with 'I'"

```rust
use oxc::allocator::Allocator;
use oxc::ast::AstKind;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;

#[derive(Debug)]
struct LintError {
    message: String,
    span: oxc::span::Span,
    suggestion: Option<String>,
}

fn lint_interfaces(source_code: &str) -> Vec<LintError> {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);

    let ret = Parser::new(&allocator, source_code, source_type).parse();
    let semantic = SemanticBuilder::new().build(&ret.program).semantic;

    let mut errors = Vec::new();

    for node in semantic.nodes().iter() {
        if let AstKind::TSInterfaceDeclaration(decl) = node.kind() {
            let name = decl.id.name.as_str();

            if name.starts_with('I') && name.chars().nth(1).map_or(false, |c| c.is_uppercase()) {
                let new_name = &name[1..];

                errors.push(LintError {
                    message: format!("Interface name '{}' should not be prefixed with 'I'", name),
                    span: decl.id.span,
                    suggestion: Some(format!("Rename to '{}'", new_name)),
                });
            }
        }
    }

    errors
}

fn main() {
    let code = r#"
        interface IUser {
            name: string;
        }
        interface Image {
            url: string;
        }
    "#;

    let violations = lint_interfaces(code);

    for v in violations {
        println!("Error: {}", v.message);
        let snippet = &code[v.span.start as usize..v.span.end as usize];
        println!("  At: '{}'", snippet);
        if let Some(sug) = v.suggestion {
            println!("  Suggestion: {}", sug);
        }
    }
}
```

### The Semantic Iterator

OXC stores nodes in a flat array (Arena) rather than a tree structure:

- **Performance**: Iterating a flat array is faster than walking a tree pointer-by-pointer
- **Convenience**: No need to manage traversal state

### Targeting the Span

Always attach lint errors to the most specific node possible:

- `decl.span`: Covers the whole interface
- `decl.id.span`: Covers only the name (preferred for error reporting)

### Context-Aware Linting

Use OXC's Scope system for advanced rules like detecting shadowed variables:

```rust
if let AstKind::VariableDeclarator(decl) = node.kind() {
    if let Some(name) = decl.id.get_identifier() {
        let scope_id = node.scope_id();
        let scopes = semantic.scopes();

        if let Some(parent_scope_id) = scopes.get_parent_id(scope_id) {
            for ancestor_scope in scopes.ancestors(parent_scope_id) {
                if scopes.get_binding(ancestor_scope, name).is_some() {
                    println!("Error: Variable '{}' shadows a variable in a parent scope.", name);
                }
            }
        }
    }
}
```

## Structured JSON Reports

Generate pipeline-ready JSON reports for integration with GitHub Actions and dashboards.

### Report Structure

```rust
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct FullReport {
    pub summary: Summary,
    pub files: HashMap<String, FileAnalysis>,
    pub dead_code: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct Summary {
    pub total_files: usize,
    pub total_functions: usize,
    pub scan_duration_ms: u128,
}

#[derive(Serialize, Debug)]
pub struct FileAnalysis {
    pub functions: Vec<String>,
    pub complexity_score: usize,
}
```

### Parallel Collection into JSON

```rust
use dashmap::DashMap;
use std::time::Instant;

fn generate_report(file_paths: Vec<PathBuf>) -> String {
    let start_time = Instant::now();
    let file_results = Arc::new(DashMap::new());
    let global_usages = Arc::new(DashSet::new());

    file_paths.par_iter().for_each(|path| {
        let allocator = Allocator::default();
        let source = std::fs::read_to_string(path).unwrap();
        let ret = Parser::new(&allocator, &source, SourceType::from_path(path).unwrap()).parse();
        let semantic = SemanticBuilder::new().build(&ret.program).semantic;

        let mut functions = Vec::new();
        for node in semantic.nodes().iter() {
            if let AstKind::Function(func) = node.kind() {
                if let Some(id) = &func.id {
                    functions.push(id.name.to_string());
                }
            }
            if let AstKind::CallExpression(call) = node.kind() {
                if let oxc::ast::ast::Expression::IdentifierReference(ident) = &call.callee {
                    global_usages.insert(ident.name.to_string());
                }
            }
        }

        file_results.insert(path.to_string_lossy().to_string(), FileAnalysis {
            functions,
            complexity_score: 0,
        });
    });

    let total_functions = file_results.iter().map(|r| r.value().functions.len()).sum();

    let dead_code = file_results.iter()
        .flat_map(|r| r.value().functions.clone())
        .filter(|f| !global_usages.contains(f))
        .collect();

    let report = FullReport {
        summary: Summary {
            total_files: file_results.len(),
            total_functions,
            scan_duration_ms: start_time.elapsed().as_millis(),
        },
        files: file_results.into_read_only().iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        dead_code,
    };

    serde_json::to_string_pretty(&report).unwrap()
}
```

### CI/CD Integration

Exit with non-zero code if dead code is found:

```rust
fn main() {
    let report_json = generate_report(paths);
    std::fs::write("analysis-report.json", &report_json).unwrap();

    let report: FullReport = serde_json::from_str(&report_json).unwrap();
    if !report.dead_code.is_empty() {
        eprintln!("FAIL: Found {} unused functions.", report.dead_code.len());
        std::process::exit(1);
    }
}
```

## WASM Integration

Compile Rust logic to WebAssembly for near-native speeds in TypeScript/browser environments.

### Project Setup

```toml
[package]
name = "oxc-wasm-tools"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
oxc = { version = "0.30.0", features = ["full"] }
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

### Rust WASM Wrapper

```rust
use wasm_bindgen::prelude::*;
use oxc::allocator::Allocator;
use oxc::ast::AstKind;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use serde::Serialize;

#[derive(Serialize)]
pub struct WasmRenameEdit {
    pub start: u32,
    pub end: u32,
    pub replacement: String,
}

#[wasm_bindgen]
pub fn get_rename_edits(source: &str, target_name: &str, new_name: &str) -> JsValue {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);

    let ret = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new().build(&ret.program).semantic;

    let mut edits = Vec::new();
    let mut target_id = None;

    // Find declaration
    for node in semantic.nodes().iter() {
        if let AstKind::TSInterfaceDeclaration(decl) = node.kind() {
            if decl.id.name == target_name {
                target_id = decl.id.symbol_id.get();
                break;
            }
        }
    }

    // Collect references
    if let Some(symbol_id) = target_id {
        let symbol_table = semantic.symbols();

        let decl_span = symbol_table.get_span(symbol_id);
        edits.push(WasmRenameEdit {
            start: decl_span.start,
            end: decl_span.end,
            replacement: new_name.to_string(),
        });

        for ref_id in symbol_table.get_resolved_references(symbol_id) {
            let reference = &semantic.references()[*ref_id];
            edits.push(WasmRenameEdit {
                start: reference.span().start,
                end: reference.span().end,
                replacement: new_name.to_string(),
            });
        }
    }

    serde_wasm_bindgen::to_value(&edits).unwrap()
}
```

### Compiling and Using

```bash
wasm-pack build --target nodejs  # or --target web
```

### TypeScript Usage

```typescript
import { get_rename_edits } from './pkg/oxc_wasm_tools';

const code = `interface IUser { id: number; } const me: IUser = { id: 1 };`;

const edits = get_rename_edits(code, "IUser", "User");

let updatedCode = code;
const sortedEdits = edits.sort((a, b) => b.start - a.start);

for (const edit of sortedEdits) {
    updatedCode =
        updatedCode.slice(0, edit.start) +
        edit.replacement +
        updatedCode.slice(edit.end);
}

console.log(updatedCode);
// Result: interface User { id: number; } const me: User = { id: 1 };
```

### When to Choose WASM

- **Speed**: 10-50x faster than pure JS for large files
- **Safety**: Rigorous TypeScript support without complex JS-based type-checkers
- **Portability**: Same `.wasm` file runs in Node.js, Deno, Bun, and browsers

## CI/CD Pipeline

### GitHub Actions Workflow

```yaml
name: Publish WASM to NPM

on:
  push:
    tags:
      - 'v*'

jobs:
  build-and-publish:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Build WASM Package
        run: wasm-pack build crates/wasm --release --target nodejs --out-dir ../../pkg

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'

      - name: Publish to NPM
        run: |
          cd pkg
          npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

## TypeScript Integration

### Parsing to JSON AST

```typescript
import { parseSync } from 'oxc-parser';

const code = `
  import { foo } from "./utils";
  const x: number = 5;
`;

const result = parseSync(code, {
  sourceFilename: 'test.ts',
  sourceType: 'module',
});

if (result.errors.length > 0) {
  console.error("Errors:", result.errors);
}

const program = JSON.parse(result.program);

program.body.forEach((node: any) => {
  if (node.type === 'ImportDeclaration') {
    console.log(`Found import from: ${node.source.value}`);
  }
  if (node.type === 'VariableDeclaration') {
    node.declarations.forEach((decl: any) => {
      if (decl.id.typeAnnotation) {
        console.log(`Type Annotation found for ${decl.id.name}`);
      }
    });
  }
});
```

### Module Resolution

```typescript
import { ResolverFactory } from 'oxc-resolver';

const resolver = new ResolverFactory({
  extensions: ['.ts', '.js', '.json'],
});

const resolution = resolver.sync(__dirname, './utils');

if (resolution.path) {
  console.log("Resolved to:", resolution.path);
}
```

### Capability Summary

| Feature | Rust Support | TypeScript/Node Support |
| --- | --- | --- |
| **AST Parsing** | Excellent (Typed, Arena-Allocated) | Good (Returns JSON) |
| **Type Info Extraction** | Excellent (Explicit types via AST) | Good (Walk JSON manually) |
| **Dependency Extraction** | Excellent (`oxc_semantic` / Visitor) | Manual (Walk JSON) |
| **Type Inference** | Alpha (Via `oxlint --type-aware`) | None (Use `tsc`) |
| **Module Resolution** | Excellent (`oxc_resolver`) | Excellent (`oxc-resolver` binding) |

## Production Best Practices

### 1. Memory Management

Always reuse your `Allocator` if processing multiple files in a loop. Allocation in OXC's arena is fast, but creating a new arena for every snippet adds overhead.

### 2. Span Management

OXC uses **byte offsets** for spans. For VS Code extensions, convert to **UTF-16 character offsets**:

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

### 3. Error Handling

Use the `miette` crate for beautiful error messages:

```rust
// OXC's diagnostics are compatible with miette
```

### 4. Bundle Size Optimization

Use `wee_alloc` to reduce WASM binary size:

```rust
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

### 5. TypeScript Definitions

Create manual `.d.ts` files for better developer experience:

```typescript
export interface WasmRenameEdit {
  start: number;
  end: number;
  replacement: string;
}

export function get_rename_edits(
  source: string,
  target_name: string,
  new_name: string
): WasmRenameEdit[];
```

## Summary

Through this guide, you've learned how to build production-ready AST analysis tools with OXC:

1. **Parsing & AST Exploration**: Leveraging the high-speed arena allocator
2. **Semantic Analysis**: Linking references to symbols for identity tracking
3. **Extraction**: Building dependency and inheritance graphs using Visitors
4. **Transformation/Linting**: Creating rules based on logical context, not just syntax
5. **Refactoring**: Performing precise renames using the Symbol Table
6. **Distribution**: Packaging into cross-platform WASM modules with CI/CD

This setup puts you in the top tier of tool builders, using the same engine that powers Oxlint, the fastest linter in the JavaScript ecosystem.

## Resources

- [OXC GitHub Repository](https://github.com/oxc-project/oxc)
- [OXC Documentation](https://oxc-project.github.io/)
- [Oxlint (Fast Linter)](https://oxc-project.github.io/docs/guide/usage/linter.html)
