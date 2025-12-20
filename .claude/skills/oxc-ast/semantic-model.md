# Semantic Model

OXC's semantic model goes beyond the AST by understanding the **meaning** of code: which identifiers refer to which declarations, what scope a variable lives in, and which references point to the same symbol.

## Key Concepts

### Symbol Table

Maps **declarations** to unique `SymbolId`s. Each symbol represents a variable, function, class, or type.

**Key Operations:**

```rust
let symbols = semantic.symbols();

// Iterate all symbols
for symbol_id in symbols.symbol_ids() {
    let name = symbols.get_name(symbol_id);
    let span = symbols.get_span(symbol_id); // Where declared
    let refs = symbols.get_resolved_references(symbol_id); // All usages

    println!("{} declared at {:?}, used {} times", name, span, refs.len());
}
```

### Scope Tree

Represents nested scopes (module, function, block, etc.). Each scope can contain bindings (variables).

**Key Operations:**

```rust
let scopes = semantic.scopes();
let root_scope = scopes.root_scope_id();

// Check parent scope
if let Some(parent_id) = scopes.get_parent_id(scope_id) {
    println!("Has parent scope");
}

// Traverse ancestors
for ancestor_scope in scopes.ancestors(scope_id) {
    // Check each parent scope
}

// Get binding in scope
if let Some(symbol_id) = scopes.get_binding(scope_id, "myVar") {
    println!("Found myVar in this scope");
}
```

### Reference Graph

Links **usages** (e.g., `console.log(x)`) to their **declarations** (e.g., `let x = 5`).

**Key Operations:**

```rust
let references = semantic.references();

for ref_id in refs {
    let reference = &references[*ref_id];
    let span = reference.span();
    println!("Referenced at {:?}", span);
}
```

## Finding Unused Symbols (Per File)

One of the most powerful uses of the semantic model is detecting unused code:

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

## Context-Aware Linting: Detecting Shadowing

Use scopes to detect when a variable shadows another variable in a parent scope:

```rust
use oxc::ast::AstKind;

for node in semantic.nodes().iter() {
    if let AstKind::VariableDeclarator(decl) = node.kind() {
        if let Some(name) = decl.id.get_identifier() {
            let scope_id = node.scope_id();
            let scopes = semantic.scopes();

            // Check parent scopes
            if let Some(parent_scope_id) = scopes.get_parent_id(scope_id) {
                for ancestor_scope in scopes.ancestors(parent_scope_id) {
                    if scopes.get_binding(ancestor_scope, name).is_some() {
                        println!("Warning: Variable '{}' shadows a variable in a parent scope", name);
                    }
                }
            }
        }
    }
}
```

## Symbol Identity vs Name Matching

**Problem:** Two files might have functions with the same name.

**Bad Approach (name matching):**

```rust
// This will confuse two different 'init()' functions!
if func_name == "init" {
    // ...
}
```

**Good Approach (symbol IDs):**

```rust
// Each symbol has a unique ID
let symbol_id = decl.id.symbol_id.get();

// Store and compare by ID, not name
declared_symbols.insert(symbol_id);
```

## Type Annotations (Explicit Types)

OXC can extract **explicit** type annotations (what's written in the code). Type inference is experimental.

```rust
use oxc::ast::ast::TSType;

if let AstKind::VariableDeclarator(decl) = node.kind() {
    if let Some(ident) = decl.id.get_identifier() {
        if let Some(type_annotation) = &decl.id.type_annotation {
            match &type_annotation.type_annotation {
                TSType::TSNumberKeyword(_) => {
                    println!("{}: number", ident);
                }
                TSType::TSStringKeyword(_) => {
                    println!("{}: string", ident);
                }
                TSType::TSTypeReference(r) => {
                    println!("{}: {:?}", ident, r.type_name);
                }
                TSType::TSArrayType(arr) => {
                    println!("{}: array", ident);
                }
                _ => {
                    println!("{}: complex type", ident);
                }
            }
        } else {
            println!("{}: no explicit type", ident);
        }
    }
}
```

## Diagnostics and Errors

OXC's semantic pass can detect logical errors (not just syntax):

```rust
let semantic_ret = SemanticBuilder::new()
    .with_check_syntax_error(true)
    .build(&ret.program);

// Parse errors
for error in ret.errors {
    println!("Syntax error: {:?}", error);
}

// Semantic errors (duplicate bindings, etc.)
for error in semantic_ret.errors {
    let error_with_source = error.with_source_code(source_code.to_string());
    println!("Semantic error: {:?}", error_with_source);
}

let semantic = semantic_ret.semantic;
```

## Common Semantic Patterns

### Find all references to a symbol

```rust
// Given a symbol_id (e.g., from a function declaration)
let symbol_table = semantic.symbols();
let refs = symbol_table.get_resolved_references(symbol_id);

for ref_id in refs {
    let reference = &semantic.references()[*ref_id];
    println!("Used at: {:?}", reference.span());
}
```

### Check if a function is exported

```rust
use oxc::ast::AstKind;

if let AstKind::Function(func) = node.kind() {
    let scope_id = node.scope_id();
    let scopes = semantic.scopes();

    // Check if this is at module scope (top-level)
    if scopes.get_parent_id(scope_id).is_none() {
        println!("Top-level function (potentially exported)");
    }
}
```

### Find all imports and what they import

```rust
use oxc::ast::ast::Statement;

for statement in &ret.program.body {
    if let Statement::ImportDeclaration(import_decl) = statement {
        println!("Import from: {}", import_decl.source.value);

        if let Some(specifiers) = &import_decl.specifiers {
            for spec in specifiers {
                println!("  - Imports: {:?}", spec.local.name);
            }
        }
    }
}
```

## When to Use Semantic vs AST Only

**Use Semantic when:**
- You need to know if two identifiers refer to the same thing
- Detecting unused code
- Building refactoring tools (rename, move)
- Context-aware linting (shadowing, scope violations)
- Finding all usages of a symbol

**Use AST only when:**
- Extracting syntax patterns (imports, exports)
- Formatting/pretty printing
- Simple structural queries
- Type annotation extraction (explicit types)
