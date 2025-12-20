# Parallel Processing with Rayon

OXC's per-thread `Allocator` design makes it perfect for parallel file processing. Use Rayon to saturate all CPU cores.

## Basic Parallel Pattern

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

    // 1. Collect all valid files
    let file_paths: Vec<PathBuf> = WalkBuilder::new(target_dir)
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|e| {
            let path = e.path();
            path.is_file() &&
            matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("ts" | "js" | "tsx" | "jsx")
            )
        })
        .map(|e| e.into_path())
        .collect();

    println!("Scanning {} files...", file_paths.len());

    // 2. Process in Parallel
    let all_functions: Vec<FuncInfo> = file_paths
        .par_iter() // Rayon parallel iterator
        .filter_map(|path| {
            total_files.fetch_add(1, Ordering::SeqCst);

            // Each thread creates its own Allocator
            let allocator = Allocator::default();
            let source_code = std::fs::read_to_string(path).ok()?;
            let source_type = SourceType::from_path(path).unwrap_or_default();

            // Parse
            let ret = Parser::new(&allocator, &source_code, source_type).parse();

            // Build Semantic Model
            let semantic = SemanticBuilder::new().build(&ret.program).semantic;

            let mut local_funcs = Vec::new();

            // Iterate nodes
            for node in semantic.nodes().iter() {
                if let AstKind::Function(func) = node.kind() {
                    let name = func.id.as_ref()
                        .map(|id| id.name.to_string())
                        .unwrap_or_else(|| "anonymous".to_string());

                    // Convert byte offset to line number
                    let line = source_code[..func.span.start as usize].lines().count();

                    local_funcs.push(FuncInfo {
                        file: path.to_string_lossy().into_owned(),
                        name,
                        line,
                    });
                }
            }

            Some(local_funcs)
        })
        .flatten() // Combine Vec<Vec<FuncInfo>> into Vec<FuncInfo>
        .collect();

    // 3. Report Results
    for func in &all_functions {
        println!("{}:{} -> function {}", func.file, func.line, func.name);
    }

    println!(
        "\nFinished. Found {} functions in {} files.",
        all_functions.len(),
        total_files.load(Ordering::SeqCst)
    );
}
```

## Key Principles

### Each Thread Gets Its Own Allocator

**Correct:**

```rust
file_paths.par_iter().for_each(|path| {
    let allocator = Allocator::default(); // Created inside the closure
    let ret = Parser::new(&allocator, source, source_type).parse();
});
```

**Incorrect (won't compile):**

```rust
let allocator = Allocator::default(); // Created once

file_paths.par_iter().for_each(|path| {
    // ERROR: Allocator is not Send
    let ret = Parser::new(&allocator, source, source_type).parse();
});
```

### Return Plain Data, Not OXC Nodes

**Correct:**

```rust
#[derive(Debug, Clone)]
struct FunctionData {
    name: String,
    line: usize,
    is_async: bool,
}

let results: Vec<FunctionData> = file_paths
    .par_iter()
    .map(|path| {
        // Extract data
        FunctionData { /* ... */ }
    })
    .collect();
```

**Incorrect:**

```rust
// DON'T try to return OXC nodes - they're tied to the Allocator
let results: Vec<&Function> = /* ... */; // Won't work
```

## Thread-Safe Aggregation with DashMap

For collecting results from multiple threads:

```rust
use dashmap::DashMap;
use std::sync::Arc;

let results = Arc::new(DashMap::new());

file_paths.par_iter().for_each(|path| {
    let allocator = Allocator::default();
    let source = std::fs::read_to_string(path).unwrap();
    let ret = Parser::new(&allocator, &source, source_type).parse();

    // Extract functions...
    let functions = vec![/* ... */];

    results.insert(path.to_string_lossy().to_string(), functions);
});

// Access results
for entry in results.iter() {
    println!("File: {}, Functions: {}", entry.key(), entry.value().len());
}
```

## Directory Traversal with `ignore`

The `ignore` crate respects `.gitignore` and is much faster than `std::fs::read_dir`:

```toml
[dependencies]
ignore = "0.4"
```

```rust
use ignore::WalkBuilder;

let file_paths: Vec<PathBuf> = WalkBuilder::new(".")
    .build()
    .filter_map(|entry| entry.ok())
    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("ts"))
    .map(|e| e.into_path())
    .collect();
```

## Performance Characteristics

**Why This is Fast:**

- **Zero-Cost Abstractions**: Rayon uses work-stealing to balance load across cores
- **Arena Allocation**: Per-thread `Allocator` means no lock contention
- **Parallel I/O**: While one thread reads, another parses

**Typical Performance:**

- **Single-threaded**: ~1000 files/second
- **8-core parallel**: ~6000-7000 files/second (6-7x speedup)

## Finding Exported Functions with Scopes

```rust
for node in semantic.nodes().iter() {
    if let AstKind::Function(func) = node.kind() {
        let scope_id = node.scope_id();
        let scopes = semantic.scopes();

        // Check if at module scope (top-level)
        if scopes.get_parent_id(scope_id).is_none() {
            println!("Top-level function: {:?}", func.id);
        }
    }
}
```

## Error Handling in Parallel

```rust
use std::sync::Mutex;

let errors = Arc::new(Mutex::new(Vec::new()));

file_paths.par_iter().for_each(|path| {
    let allocator = Allocator::default();

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            errors.lock().unwrap().push(format!("{}: {}", path.display(), e));
            return;
        }
    };

    let ret = Parser::new(&allocator, &source, source_type).parse();

    if !ret.errors.is_empty() {
        errors.lock().unwrap().push(format!("{}: parse errors", path.display()));
    }

    // Continue processing...
});

// Report errors
for error in errors.lock().unwrap().iter() {
    eprintln!("{}", error);
}
```

## Combining with Other Tools

### With `oxc_resolver` for Module Resolution

```rust
use oxc_resolver::{ResolveOptions, Resolver};

let resolver = Resolver::new(ResolveOptions::default());

file_paths.par_iter().for_each(|path| {
    // ... parse file ...

    // Resolve imports
    for import in imports {
        if let Ok(resolved) = resolver.resolve(path.parent().unwrap(), import) {
            println!("Resolved: {} -> {}", import, resolved.path().display());
        }
    }
});
```

## CLI Pattern Summary

1. **Discover files** using `ignore` (respects `.gitignore`)
2. **Parallelize** using Rayon's `par_iter`
3. **Encapsulate** `Allocator` and `Parser` inside each thread
4. **Extract** plain data structures (not OXC nodes)
5. **Aggregate** results using thread-safe containers or `collect()`
6. **Report** findings or write to files

## Production Checklist

- [ ] Use `ignore` crate for file discovery
- [ ] Create `Allocator` per thread
- [ ] Extract plain data, not OXC nodes
- [ ] Handle parse errors gracefully
- [ ] Use atomic counters for progress tracking
- [ ] Consider memory limits for very large codebases (process in batches if needed)
