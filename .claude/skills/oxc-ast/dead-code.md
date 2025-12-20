# Dead Code Detection

Detecting unused functions requires **two-pass analysis** across the entire codebase: first collect all declarations and usages, then reconcile them to find symbols that are never used.

## The Two-Pass Strategy

1. **Pass 1 (Parallel)**: Scan every file to collect all **Declarations** (symbols) and all **Usages** (references)
2. **Pass 2 (Global)**: Compare declarations against usages to find dead code

## Global Dead Code Detector

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
    // Functions that are declared (FilePath:FunctionName)
    declared_functions: DashSet<String>,
    // Functions that are called
    used_functions: DashSet<String>,
}

fn main() {
    let detector = Arc::new(DeadCodeDetector {
        declared_functions: DashSet::new(),
        used_functions: DashSet::new(),
    });

    let file_paths = vec![/* ... paths ... */];

    // --- PASS 1: Collection ---
    file_paths.par_iter().for_each(|path| {
        let allocator = Allocator::default();
        let source = std::fs::read_to_string(path).unwrap();
        let ret = Parser::new(&allocator, &source, SourceType::from_path(path).unwrap()).parse();
        let semantic = SemanticBuilder::new().build(&ret.program).semantic;

        for node in semantic.nodes().iter() {
            match node.kind() {
                // Find Declarations
                AstKind::Function(func) => {
                    if let Some(id) = &func.id {
                        detector.declared_functions.insert(id.name.to_string());
                    }
                }
                // Find Calls (Usages)
                AstKind::CallExpression(call) => {
                    if let oxc::ast::ast::Expression::IdentifierReference(ident) = &call.callee {
                        detector.used_functions.insert(ident.name.to_string());
                    }
                }
                _ => {}
            }
        }
    });

    // --- PASS 2: Reconciliation ---
    println!("--- Potentially Unused Functions ---");
    for func in detector.declared_functions.iter() {
        if !detector.used_functions.contains(func.key()) {
            println!("Unused: {}", func.key());
        }
    }
}
```

## Handling False Positives

The naive approach above has a major flaw: it doesn't account for **exported functions**. A function might be exported and used by external consumers.

### Refinements Needed

1. **Identify Exports**: Check if the function is wrapped in an `ExportNamedDeclaration`
2. **Trace Imports**: Use `oxc_resolver` to follow imports and verify if imported symbols are called
3. **Entry Points**: Define entry points (like `index.ts` or `main.ts`). Any function not reachable via the call graph from these entries is dead

### Checking for Exports

```rust
use oxc::ast::ast::Statement;

for statement in &ret.program.body {
    match statement {
        Statement::ExportNamedDeclaration(export_decl) => {
            // This function is exported - don't mark as dead
            if let Some(decl) = &export_decl.declaration {
                // Extract function name from declaration
                // Mark as "exported"
            }
        }
        _ => {}
    }
}
```

## Using Symbol IDs for Accuracy

To avoid name collisions (e.g., two files with `init()` functions), use **unique Symbol IDs**:

```rust
use std::collections::HashSet;

let mut declared_symbols: HashSet<oxc::semantic::SymbolId> = HashSet::new();
let mut used_symbols: HashSet<oxc::semantic::SymbolId> = HashSet::new();

for node in semantic.nodes().iter() {
    if let AstKind::Function(func) = node.kind() {
        if let Some(symbol_id) = func.id.as_ref().and_then(|id| id.symbol_id.get()) {
            declared_symbols.insert(symbol_id);
        }
    }
}
```

## Per-File Unused Detection

For a single file, use the semantic model's built-in reference tracking:

```rust
let symbols = semantic.symbols();

for symbol_id in symbols.symbol_ids() {
    let name = symbols.get_name(symbol_id);
    let refs = symbols.get_resolved_references(symbol_id);

    if refs.is_empty() {
        println!("Symbol '{}' is never used in this file", name);
    }
}
```

## Cross-File Dead Code Analysis

For a robust multi-file analysis:

1. **Build Global Symbol Table**: Map `(file, symbol_id) -> name`
2. **Track Module Imports**: Use `oxc_resolver` to resolve import paths
3. **Build Call Graph**: Track which files call which symbols
4. **Mark Reachable**: Starting from entry points, mark all reachable symbols
5. **Report Unreachable**: Any symbol not marked is dead code

```rust
use std::collections::{HashMap, HashSet};

struct GlobalAnalyzer {
    // Map: (file, symbol_id) -> name
    symbols: HashMap<(String, oxc::semantic::SymbolId), String>,
    // Map: symbol -> files that import it
    imports: HashMap<String, HashSet<String>>,
    // Reachable symbols
    reachable: HashSet<String>,
}

impl GlobalAnalyzer {
    fn mark_reachable(&mut self, entry_points: &[String]) {
        // Start from entry points and traverse the call graph
        // Mark all symbols reachable from entry points
    }

    fn find_dead_code(&self) -> Vec<String> {
        self.symbols
            .iter()
            .filter(|((_, _), name)| !self.reachable.contains(*name))
            .map(|((file, _), name)| format!("{}:{}", file, name))
            .collect()
    }
}
```

## Why OXC is the Right Tool

- **Memory Efficiency**: Using `DashSet` with OXC's arena-allocated nodes, you can analyze millions of lines with minimal memory
- **Accuracy**: OXC understands TypeScript scopes perfectly, avoiding false positives from name collisions
- **Speed**: Parallel processing with Rayon makes whole-codebase analysis feasible in seconds

## Integration with CI/CD

Generate a report and fail the build if dead code is found:

```rust
fn main() {
    let dead_code = find_dead_code(&file_paths);

    if !dead_code.is_empty() {
        eprintln!("FAIL: Found {} unused functions:", dead_code.len());
        for func in &dead_code {
            eprintln!("  - {}", func);
        }
        std::process::exit(1);
    }

    println!("OK: No dead code detected");
}
```

See [JSON Reporting](./json-reporting.md) for structured output formats.
