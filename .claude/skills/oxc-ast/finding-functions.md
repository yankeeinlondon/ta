# Finding Functions

JavaScript/TypeScript has three main ways to define functions: **declarations**, **expressions**, and **arrow functions**. OXC's visitor pattern handles all of them.

## The Multi-Pattern Function Visitor

```rust
use oxc::allocator::Allocator;
use oxc::ast::ast::Function;
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
    // Handles: function foo() {} AND const x = function() {}
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

    // Handles: () => {}
    fn visit_arrow_function_expression(
        &mut self,
        func: &oxc::ast::ast::ArrowFunctionExpression<'a>
    ) {
        self.functions.push(FunctionMetadata {
            name: "arrow".to_string(),
            is_async: func.r#async,
            is_generator: false, // Arrows cannot be generators
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

## AST Node Types for Functions

| Function Type | AST Node | Common Properties |
|---------------|----------|-------------------|
| **Declaration** | `Function` | `id` is present, `kind` is `FunctionDeclaration` |
| **Expression** | `Function` | `id` may be null, `kind` is `FunctionExpression` |
| **Arrow** | `ArrowFunctionExpression` | No `id`, `expression` boolean for shorthand |
| **Class Method** | `MethodDefinition` | Contains a `Function` inside its `value` property |

## Finding Class Methods

To find methods inside classes, add this to your visitor:

```rust
fn visit_method_definition(&mut self, it: &oxc::ast::ast::MethodDefinition<'a>) {
    let name = it.key.static_name().unwrap_or("dynamic_method".into());
    println!("Found method: {}", name);

    // The actual function body/params are in it.value
    walk::walk_method_definition(self, it);
}
```

## Using Semantic Iteration (Alternative)

Instead of a visitor, you can iterate through semantic nodes:

```rust
use oxc::semantic::SemanticBuilder;
use oxc::ast::AstKind;

let semantic = SemanticBuilder::new().build(&ret.program).semantic;

for node in semantic.nodes().iter() {
    match node.kind() {
        AstKind::Function(func) => {
            let name = func.id.as_ref()
                .map(|id| id.name.to_string())
                .unwrap_or_else(|| "anonymous".to_string());

            println!("Function: {}", name);
        }
        AstKind::ArrowFunctionExpression(arrow) => {
            println!("Arrow function with {} params", arrow.params.items.len());
        }
        _ => {}
    }
}
```

## Extracting Line Numbers

To report where a function is defined:

```rust
let line = source_code[..func.span.start as usize].lines().count();
println!("Function '{}' at line {}", name, line);
```

## Finding Exported Functions

Check if a function is at module scope (top-level):

```rust
use oxc::semantic::SemanticBuilder;

let semantic = SemanticBuilder::new().build(&ret.program).semantic;

for node in semantic.nodes().iter() {
    if let AstKind::Function(func) = node.kind() {
        let scope_id = node.scope_id();
        let scopes = semantic.scopes();

        // Check if the scope is the top-level (module) scope
        if scopes.get_parent_id(scope_id).is_none() {
            println!("Top-level function: {:?}", func.id);
        }
    }
}
```

## Performance Considerations

When scanning large codebases:

1. **Use Rayon** for parallel file processing (see [Parallel Processing](./parallel-processing.md))
2. **Create Allocator per thread** - OXC's allocator is not `Send`
3. **Return plain data** - Extract `FunctionMetadata` structs, not OXC nodes
4. **Aggregate results** - Use thread-safe containers like `DashMap` or collect results at the end

```rust
use rayon::prelude::*;
use std::sync::Arc;
use dashmap::DashMap;

let results = Arc::new(DashMap::new());

file_paths.par_iter().for_each(|path| {
    let allocator = Allocator::default();
    let source = std::fs::read_to_string(path).unwrap();
    let ret = Parser::new(&allocator, &source, source_type).parse();

    let mut finder = FunctionFinder { functions: vec![] };
    finder.visit_program(&ret.program);

    results.insert(path.to_string_lossy().to_string(), finder.functions);
});
```

## Refining with Semantic Analysis

Use the Symbol Table to find where a function is **called** versus where it is **defined**:

```rust
let symbols = semantic.symbols();

for symbol_id in symbols.symbol_ids() {
    let name = symbols.get_name(symbol_id);
    let refs = symbols.get_resolved_references(symbol_id);

    println!("Function '{}' called {} times", name, refs.len());

    for ref_id in refs {
        let reference = &semantic.references()[*ref_id];
        println!("  Called at: {:?}", reference.span());
    }
}
```
