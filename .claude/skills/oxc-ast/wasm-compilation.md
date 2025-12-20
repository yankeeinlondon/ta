# WASM Compilation

Compile your Rust OXC analysis logic to WebAssembly for use in Node.js, browsers, and VS Code extensions.

## Setup

**Cargo.toml:**

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

## Basic WASM Export

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

    // 1. Find declaration
    for node in semantic.nodes().iter() {
        if let AstKind::TSInterfaceDeclaration(decl) = node.kind() {
            if decl.id.name == target_name {
                target_id = decl.id.symbol_id.get();
                break;
            }
        }
    }

    // 2. Collect references
    if let Some(symbol_id) = target_id {
        let symbol_table = semantic.symbols();

        // Add declaration
        let decl_span = symbol_table.get_span(symbol_id);
        edits.push(WasmRenameEdit {
            start: decl_span.start,
            end: decl_span.end,
            replacement: new_name.to_string(),
        });

        // Add all usages
        for ref_id in symbol_table.get_resolved_references(symbol_id) {
            let reference = &semantic.references()[*ref_id];
            edits.push(WasmRenameEdit {
                start: reference.span().start,
                end: reference.span().end,
                replacement: new_name.to_string(),
            });
        }
    }

    // Convert Rust Vec to JavaScript Value
    serde_wasm_bindgen::to_value(&edits).unwrap()
}
```

## Building

**For Node.js:**

```bash
wasm-pack build --target nodejs
```

**For Web:**

```bash
wasm-pack build --target web
```

**For Bundlers (webpack, vite):**

```bash
wasm-pack build --target bundler
```

## TypeScript Usage

**Node.js:**

```typescript
import { get_rename_edits } from './pkg/oxc_wasm_tools';

const code = `interface IUser { id: number; } const me: IUser = { id: 1 };`;

// Call the Rust function
const edits = get_rename_edits(code, "IUser", "User");

// Apply edits in reverse order to maintain index integrity
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

**Web:**

```html
<script type="module">
import init, { get_rename_edits } from './pkg/oxc_wasm_tools.js';

await init();

const code = `interface IUser { id: number; }`;
const edits = get_rename_edits(code, "IUser", "User");
console.log(edits);
</script>
```

## Advanced: Returning Complex Types

**Problem:** You can't directly return OXC nodes or complex Rust types to JavaScript.

**Solution:** Serialize to JSON or create simple data structures.

```rust
#[derive(Serialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub properties: Vec<PropertyInfo>,
}

#[derive(Serialize)]
pub struct PropertyInfo {
    pub name: String,
    pub type_name: String,
}

#[wasm_bindgen]
pub fn extract_interfaces(source: &str) -> JsValue {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = Parser::new(&allocator, source, source_type).parse();

    let mut interfaces = Vec::new();

    // ... extract interface data ...

    serde_wasm_bindgen::to_value(&interfaces).unwrap()
}
```

## Error Handling

```rust
#[wasm_bindgen]
pub fn analyze_code(source: &str) -> Result<JsValue, JsError> {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = Parser::new(&allocator, source, source_type).parse();

    if !ret.errors.is_empty() {
        return Err(JsError::new(&format!("Parse errors: {}", ret.errors.len())));
    }

    // ... analysis ...

    Ok(serde_wasm_bindgen::to_value(&result)?)
}
```

**TypeScript usage:**

```typescript
try {
    const result = analyze_code(source);
    console.log(result);
} catch (error) {
    console.error("Analysis failed:", error.message);
}
```

## Optimization

**Cargo.toml profile:**

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link Time Optimization
codegen-units = 1   # Better optimization
strip = true        # Remove debug symbols
```

**Reduce bundle size with wee_alloc:**

```toml
[dependencies]
wee_alloc = "0.4"
```

```rust
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

## Performance Considerations

**When to Use WASM:**

- Processing large files (10,000+ lines) in a browser-based IDE
- Real-time linting in VS Code extensions
- Build tools that need maximum speed

**Performance Gains:**

- 10-50x faster than pure JavaScript AST walkers
- Near-native speed for parsing and analysis
- Minimal memory overhead

## VS Code Extension Integration

**extension.ts:**

```typescript
import * as vscode from 'vscode';
import { get_rename_edits } from '../pkg/oxc_wasm_tools';

export function activate(context: vscode.ExtensionContext) {
    const renameProvider = vscode.languages.registerRenameProvider(
        { scheme: 'file', language: 'typescript' },
        {
            provideRenameEdits(document, position, newName) {
                const source = document.getText();
                const offset = document.offsetAt(position);

                // Find symbol at position...
                const oldName = getSymbolAtOffset(source, offset);

                // Get edits from Rust
                const edits = get_rename_edits(source, oldName, newName);

                // Convert to VS Code edits
                const workspaceEdit = new vscode.WorkspaceEdit();

                for (const edit of edits) {
                    const start = document.positionAt(edit.start);
                    const end = document.positionAt(edit.end);
                    workspaceEdit.replace(
                        document.uri,
                        new vscode.Range(start, end),
                        edit.replacement
                    );
                }

                return workspaceEdit;
            }
        }
    );

    context.subscriptions.push(renameProvider);
}
```

## TypeScript Definitions

`wasm-pack` generates `.d.ts` files automatically:

```typescript
/* tslint:disable */
/* eslint-disable */
/**
* @param {string} source
* @param {string} target_name
* @param {string} new_name
* @returns {any}
*/
export function get_rename_edits(source: string, target_name: string, new_name: string): any;
```

**Improve with manual definitions:**

```typescript
export interface RenameEdit {
    start: number;
    end: number;
    replacement: string;
}

export function get_rename_edits(
    source: string,
    target_name: string,
    new_name: string
): RenameEdit[];
```

## Debugging WASM

**Enable console logging:**

```toml
[dependencies]
console_error_panic_hook = "0.1"
```

```rust
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
```

**TypeScript:**

```typescript
// Now panics will show in browser console
import init from './pkg/oxc_wasm_tools.js';
await init();
```

## Package.json for NPM

```json
{
  "name": "oxc-wasm-tools",
  "version": "0.1.0",
  "files": [
    "pkg/"
  ],
  "main": "pkg/oxc_wasm_tools.js",
  "types": "pkg/oxc_wasm_tools.d.ts",
  "scripts": {
    "build": "wasm-pack build --release --target nodejs",
    "build:web": "wasm-pack build --release --target web"
  }
}
```

## When to Use WASM

**Good Use Cases:**

- Browser-based IDEs (CodeSandbox, StackBlitz)
- VS Code extensions needing high performance
- CLI tools distributed via npm (without Rust dependency)
- Real-time code analysis in web apps

**Not Ideal For:**

- Simple one-off scripts (Rust CLI is faster to build)
- Server-side processing (native Rust is faster)
- Very small codebases (overhead of WASM initialization)

## UTF-8 vs UTF-16 Spans

**Critical:** OXC uses byte offsets (UTF-8). JavaScript uses character offsets (UTF-16).

**Convert for JavaScript:**

```rust
#[wasm_bindgen]
pub fn get_line_column(source: &str, byte_offset: u32) -> JsValue {
    let mut line = 0;
    let mut col = 0;

    for (i, c) in source.char_indices() {
        if i >= byte_offset as usize { break; }

        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    serde_wasm_bindgen::to_value(&(line, col)).unwrap()
}
```

## Summary

| Component | Rust | WASM | JS/TS |
|-----------|------|------|-------|
| **Parsing** | Rust (OXC) | Compiled to WASM | Call via binding |
| **Analysis** | Rust (Visitors) | Compiled to WASM | Receive results |
| **Data Transfer** | Serde Serialize | JSON serialization | Parse JSON/objects |
| **Errors** | Rust Result | JsError | try/catch |
