# Custom Lint Rules

OXC's semantic model enables **context-aware linting** that goes beyond syntax patterns. You can write rules that understand scopes, symbols, and references.

## Lint Rule Architecture

1. **Parser**: Creates the AST
2. **Semantic Builder**: Resolves scopes and symbols
3. **Linter Pass**: Queries the semantic model and returns diagnostics

## Example: "No I-Prefix" Rule

This rule bans interfaces starting with `I` (e.g., `IUser` → `User`).

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

    // 1. Parse
    let ret = Parser::new(&allocator, source_code, source_type).parse();

    // 2. Build Semantic Model
    let semantic = SemanticBuilder::new().build(&ret.program).semantic;

    let mut errors = Vec::new();

    // 3. Iterate Semantic Nodes
    for node in semantic.nodes().iter() {
        if let AstKind::TSInterfaceDeclaration(decl) = node.kind() {
            let name = decl.id.name.as_str();

            // 4. Check if starts with 'I' followed by uppercase
            if name.starts_with('I') && name.chars().nth(1).map_or(false, |c| c.is_uppercase()) {
                let new_name = &name[1..];

                errors.push(LintError {
                    message: format!("Interface '{}' should not be prefixed with 'I'", name),
                    span: decl.id.span, // Attach to the identifier, not the whole block
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

        interface IAdmin extends IUser {
            permissions: string[];
        }
    "#;

    let violations = lint_interfaces(code);

    println!("Found {} violations:\n", violations.len());

    for v in violations {
        println!("Error: {}", v.message);
        let snippet = &code[v.span.start as usize..v.span.end as usize];
        println!("  At: '{}'", snippet);
        if let Some(sug) = v.suggestion {
            println!("  Suggestion: {}", sug);
        }
        println!("---");
    }
}
```

## Targeting the Right Span

Always attach errors to the **most specific** node:

- `decl.span`: Entire interface (`interface IUser { ... }`)
- `decl.id.span`: Just the name (`IUser`) ← **Best practice**

```rust
errors.push(LintError {
    span: decl.id.span, // NOT decl.span
    // ...
});
```

## Context-Aware Linting: Detecting Shadowing

Use scopes to detect variable shadowing:

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
                        println!("Error: Variable '{}' shadows a variable in a parent scope", name);
                    }
                }
            }
        }
    }
}
```

## Performance: Semantic Iteration vs Tree Walking

OXC stores nodes in a **flat array** (arena), not a tree.

**Fast (Semantic Iteration):**

```rust
for node in semantic.nodes().iter() {
    if let AstKind::TSInterfaceDeclaration(decl) = node.kind() {
        // Process
    }
}
```

**Slower (Recursive Tree Walking):**

```rust
// Don't do this - walking pointers is slower than iterating a flat array
fn walk_tree(node: &Node) {
    // Recursive traversal
    for child in node.children() {
        walk_tree(child);
    }
}
```

## Example: Ban Unused Variables

```rust
let symbols = semantic.symbols();

for symbol_id in symbols.symbol_ids() {
    let name = symbols.get_name(symbol_id);
    let refs = symbols.get_resolved_references(symbol_id);

    if refs.is_empty() {
        let span = symbols.get_span(symbol_id);
        errors.push(LintError {
            message: format!("Variable '{}' is declared but never used", name),
            span,
            suggestion: Some(format!("Remove unused variable '{}'", name)),
        });
    }
}
```

## Example: Enforce Const Over Let

```rust
use oxc::ast::ast::VariableDeclarationKind;

for node in semantic.nodes().iter() {
    if let AstKind::VariableDeclaration(decl) = node.kind() {
        if decl.kind == VariableDeclarationKind::Let {
            // Check if the variable is never reassigned
            for declarator in &decl.declarations {
                if let Some(id) = declarator.id.get_identifier() {
                    if let Some(symbol_id) = declarator.id.symbol_id.get() {
                        let refs = symbols.get_resolved_references(symbol_id);

                        // Check if any reference is a write
                        let has_write = refs.iter().any(|ref_id| {
                            semantic.references()[*ref_id].is_write()
                        });

                        if !has_write {
                            errors.push(LintError {
                                message: format!("'{}' is never reassigned, use 'const' instead", id),
                                span: declarator.id.span,
                                suggestion: Some("Change to 'const'".to_string()),
                            });
                        }
                    }
                }
            }
        }
    }
}
```

## Integration with miette for Pretty Errors

```rust
use miette::{Diagnostic, SourceSpan};

#[derive(Debug, Diagnostic)]
#[diagnostic(code(lint::no_i_prefix))]
struct NoIPrefixError {
    #[source_code]
    src: String,

    #[label("Remove 'I' prefix")]
    span: SourceSpan,

    #[help]
    help: String,
}

fn report_error(source: &str, error: &LintError) {
    let diagnostic = NoIPrefixError {
        src: source.to_string(),
        span: (error.span.start as usize, (error.span.end - error.span.start) as usize).into(),
        help: error.suggestion.clone().unwrap_or_default(),
    };

    eprintln!("{:?}", miette::Report::new(diagnostic));
}
```

## TypeScript/Node.js Linting

If using `oxc-parser` in Node.js (JSON AST):

```typescript
import { parseSync } from 'oxc-parser';

const source = `interface IUser { id: number; }`;
const ret = parseSync(source, { sourceFilename: 'test.ts' });
const program = JSON.parse(ret.program);

function walk(node: any) {
    if (node.type === 'TSInterfaceDeclaration') {
        const name = node.id.name;
        if (name.startsWith('I') && /^[A-Z]/.test(name.slice(1))) {
            console.log(`Violation: ${name} starts with I`);
        }
    }

    // Recursively walk children
    for (const key in node) {
        if (typeof node[key] === 'object' && node[key] !== null) {
            if (Array.isArray(node[key])) {
                node[key].forEach(walk);
            } else {
                walk(node[key]);
            }
        }
    }
}

walk(program);
```

## Building a Linter CLI

```rust
use std::path::PathBuf;
use rayon::prelude::*;

fn main() {
    let file_paths: Vec<PathBuf> = /* ... */;
    let mut total_errors = 0;

    file_paths.par_iter().for_each(|path| {
        let source = std::fs::read_to_string(path).unwrap();
        let errors = lint_interfaces(&source);

        for error in &errors {
            eprintln!("{}:{}: {}", path.display(), error.span.start, error.message);
        }

        total_errors += errors.len();
    });

    if total_errors > 0 {
        eprintln!("\nFound {} total errors", total_errors);
        std::process::exit(1);
    }
}
```

## Why OXC for Linting

- **Fast**: 10-50x faster than ESLint on large codebases
- **Accurate**: Semantic analysis prevents false positives
- **Flexible**: Write rules in Rust (performance) or consume via WASM (portability)
- **Type-Aware**: Understands TypeScript types natively
