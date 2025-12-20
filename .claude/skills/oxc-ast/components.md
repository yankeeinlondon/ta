# OXC Components

OXC differs from other JavaScript tooling because it prioritizes performance through **memory arenas** and a **semantic analysis pass** that pre-calculates scopes, symbols, and references.

## Core Components

### `oxc_parser` - The Parser

Generates the AST from source text. Allocates nodes into a memory arena (`oxc_allocator`), making allocation/deallocation extremely fast.

**Key Features:**
- Supports TypeScript, JSX, and latest ECMAScript
- Returns parse errors without crashing
- Uses byte offsets (UTF-8) for spans

**Usage:**

```rust
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::span::SourceType;

let allocator = Allocator::default();
let source_type = SourceType::from_path("file.ts").unwrap();
let ret = Parser::new(&allocator, source_code, source_type).parse();

if !ret.errors.is_empty() {
    eprintln!("Parse errors: {:?}", ret.errors);
}
```

### `oxc_ast` - The AST Definitions

Strictly typed AST similar to ESTree but with better type safety.

**Key Distinctions:**
- `BindingIdentifier` vs `IdentifierReference` (declaration vs usage)
- Separate types for `FunctionDeclaration` vs `FunctionExpression`
- TypeScript nodes (e.g., `TSInterfaceDeclaration`, `TSTypeAnnotation`)

**Pattern:**

```rust
use oxc::ast::AstKind;

match node.kind() {
    AstKind::Function(func) => { /* handle function */ },
    AstKind::TSInterfaceDeclaration(decl) => { /* handle interface */ },
    _ => {}
}
```

### `oxc_semantic` - The Semantic Analyzer

The "brain" of OXC. Consumes the AST and produces a `Semantic` struct containing:

**Symbol Table:** All variables/functions declared with unique `SymbolId`s
**Scope Tree:** Nested scopes (block, function, module)
**Reference Graph:** Links usages (`x`) to declarations (`let x`)

**Usage:**

```rust
use oxc::semantic::SemanticBuilder;

let semantic = SemanticBuilder::new().build(&ret.program).semantic;

// Access symbols
let symbols = semantic.symbols();
for symbol_id in symbols.symbol_ids() {
    let name = symbols.get_name(symbol_id);
    let refs = symbols.get_resolved_references(symbol_id);
    println!("{} used {} times", name, refs.len());
}
```

## Processing Pipeline

**Standard Flow:**

1. **Parse** - Convert source text to AST
2. **Semantic Analysis** - Build symbol table, scopes, and references
3. **Analysis/Transformation** - Use visitor pattern or node iteration
4. **Output** - Generate reports, edits, or diagnostics

```rust
// Complete example
let allocator = Allocator::default();
let source_type = SourceType::default().with_typescript(true);

// Step 1: Parse
let ret = Parser::new(&allocator, source_code, source_type).parse();

// Step 2: Semantic
let semantic = SemanticBuilder::new().build(&ret.program).semantic;

// Step 3: Analyze
for node in semantic.nodes().iter() {
    // Your logic here
}
```

## Error Handling

### Parse Errors

```rust
for error in ret.errors {
    println!("Syntax error: {:?}", error);
}
```

### Semantic Errors

```rust
let semantic_ret = SemanticBuilder::new()
    .with_check_syntax_error(true)
    .build(&ret.program);

for error in semantic_ret.errors {
    let error_with_source = error.with_source_code(source_code.to_string());
    println!("{:?}", error_with_source);
}
```

## Memory Management

**Key Insight:** OXC's `Allocator` is **not** `Send` (thread-safe). You must create one per thread.

**Correct Pattern:**

```rust
use rayon::prelude::*;

file_paths.par_iter().for_each(|path| {
    // Each thread gets its own allocator
    let allocator = Allocator::default();
    let source = std::fs::read_to_string(path).unwrap();
    let ret = Parser::new(&allocator, &source, source_type).parse();
    // ... analyze ...
});
```

**Incorrect Pattern (won't compile):**

```rust
let allocator = Allocator::default(); // Created once

file_paths.par_iter().for_each(|path| {
    // ERROR: Cannot share allocator across threads
    let ret = Parser::new(&allocator, source, source_type).parse();
});
```

## Capability Matrix

| Feature | Rust Support | TypeScript/Node Support |
|---------|-------------|------------------------|
| **AST Parsing** | Excellent (Typed, Arena-Allocated) | Good (Returns JSON) |
| **Type Info Extraction** | Excellent (Explicit types via AST) | Good (Walk JSON manually) |
| **Dependency Extraction** | Excellent (`oxc_semantic` / Visitor) | Manual (Walk JSON) |
| **Type Inference** | Alpha (Via `oxlint --type-aware`) | None (Use `tsc`) |
| **Module Resolution** | Excellent (`oxc_resolver`) | Excellent (`oxc-resolver` binding) |
