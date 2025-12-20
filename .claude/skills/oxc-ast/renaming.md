# Symbol Renaming

OXC's semantic model makes precise symbol renaming possible by tracking **all** references to a symbol, including its declaration.

## Basic Renaming Pattern

1. Find the declaration and get its `SymbolId`
2. Get all references to that symbol
3. Generate edit operations for the declaration and all references
4. Apply edits in reverse order (to maintain span integrity)

## Complete Renaming Example

```rust
use oxc::allocator::Allocator;
use oxc::ast::AstKind;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;

#[derive(Debug)]
struct RenameEdit {
    start: u32,
    end: u32,
    replacement: String,
}

fn get_rename_edits(source: &str, target_name: &str, new_name: &str) -> Vec<RenameEdit> {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);

    let ret = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new().build(&ret.program).semantic;

    let mut edits = Vec::new();
    let mut target_id = None;

    // Step 1: Find the declaration
    for node in semantic.nodes().iter() {
        if let AstKind::TSInterfaceDeclaration(decl) = node.kind() {
            if decl.id.name == target_name {
                target_id = decl.id.symbol_id.get();
                break;
            }
        }
    }

    // Step 2: Collect all references
    if let Some(symbol_id) = target_id {
        let symbol_table = semantic.symbols();

        // Add declaration edit
        let decl_span = symbol_table.get_span(symbol_id);
        edits.push(RenameEdit {
            start: decl_span.start,
            end: decl_span.end,
            replacement: new_name.to_string(),
        });

        // Add all usage edits
        for ref_id in symbol_table.get_resolved_references(symbol_id) {
            let reference = &semantic.references()[*ref_id];
            edits.push(RenameEdit {
                start: reference.span().start,
                end: reference.span().end,
                replacement: new_name.to_string(),
            });
        }
    }

    edits
}

fn apply_edits(source: &str, edits: Vec<RenameEdit>) -> String {
    // Sort edits in reverse order to maintain index integrity
    let mut sorted_edits = edits;
    sorted_edits.sort_by(|a, b| b.start.cmp(&a.start));

    let mut result = source.to_string();

    for edit in sorted_edits {
        result = format!(
            "{}{}{}",
            &result[..edit.start as usize],
            edit.replacement,
            &result[edit.end as usize..]
        );
    }

    result
}

fn main() {
    let code = r#"
        interface IUser {
            id: number;
        }

        const me: IUser = { id: 1 };
        const you: IUser = { id: 2 };
    "#;

    let edits = get_rename_edits(code, "IUser", "User");
    let updated_code = apply_edits(code, edits);

    println!("{}", updated_code);
    // Result: interface User { id: number; } const me: User = ...
}
```

## Renaming Functions

```rust
for node in semantic.nodes().iter() {
    if let AstKind::Function(func) = node.kind() {
        if let Some(id) = &func.id {
            if id.name == target_name {
                target_symbol_id = id.symbol_id.get();
                break;
            }
        }
    }
}
```

## Renaming Variables

```rust
for node in semantic.nodes().iter() {
    if let AstKind::VariableDeclarator(decl) = node.kind() {
        if let Some(name) = decl.id.get_identifier() {
            if name == target_name {
                target_symbol_id = decl.id.symbol_id.get();
                break;
            }
        }
    }
}
```

## Cross-File Renaming

For multi-file refactoring:

1. **Find all files** that might reference the symbol
2. **Parse each file** and build semantic models
3. **Track imports** to find which files import the symbol
4. **Generate edits** for each file
5. **Apply edits** atomically or return all edits for user confirmation

```rust
use std::collections::HashMap;

struct CrossFileRenamer {
    // Map: file_path -> Vec<RenameEdit>
    edits_by_file: HashMap<String, Vec<RenameEdit>>,
}

impl CrossFileRenamer {
    fn rename_across_files(&mut self, files: &[String], old_name: &str, new_name: &str) {
        for file_path in files {
            let source = std::fs::read_to_string(file_path).unwrap();
            let edits = get_rename_edits(&source, old_name, new_name);

            if !edits.is_empty() {
                self.edits_by_file.insert(file_path.clone(), edits);
            }
        }
    }

    fn apply_all(&self) {
        for (file_path, edits) in &self.edits_by_file {
            let source = std::fs::read_to_string(file_path).unwrap();
            let updated = apply_edits(&source, edits.clone());
            std::fs::write(file_path, updated).unwrap();
        }
    }
}
```

## WASM Integration for VS Code

Export the rename functionality to JavaScript for use in editors:

```rust
use wasm_bindgen::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct WasmRenameEdit {
    pub start: u32,
    pub end: u32,
    pub replacement: String,
}

#[wasm_bindgen]
pub fn get_rename_edits_wasm(
    source: &str,
    target_name: &str,
    new_name: &str
) -> JsValue {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);

    let ret = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new().build(&ret.program).semantic;

    let mut edits = Vec::new();
    // ... (same logic as above)

    serde_wasm_bindgen::to_value(&edits).unwrap()
}
```

**TypeScript usage:**

```typescript
import { get_rename_edits_wasm } from './pkg/oxc_tools';

const code = `interface IUser { id: number; }`;
const edits = get_rename_edits_wasm(code, "IUser", "User");

// Apply edits in reverse order
const sorted = edits.sort((a, b) => b.start - a.start);
let updated = code;

for (const edit of sorted) {
    updated = updated.slice(0, edit.start) + edit.replacement + updated.slice(edit.end);
}

console.log(updated);
```

## Span Accuracy

**Important:** OXC returns byte offsets (UTF-8). For JavaScript/VS Code (which uses UTF-16), you must convert:

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

## Safety Considerations

1. **Always use SymbolId, not names** - Prevents accidental renames of unrelated symbols with the same name
2. **Sort edits in reverse** - Applying edits from end to start maintains span integrity
3. **Validate before applying** - Check that the target symbol exists before generating edits
4. **Handle scope correctly** - Don't rename symbols in different scopes with the same name

## Why OXC is the Right Tool

- **Accuracy**: Semantic model ensures you rename the exact symbol, not just text matches
- **Performance**: Fast enough to compute rename previews in real-time for VS Code extensions
- **Type Safety**: Rust's type system prevents many common refactoring bugs
