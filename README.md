# TA (TypeScript Analyzer)

> A speedy AST analyzer written in Rust using OXC (Oxidation Compiler)

**Status:** Core functionality implemented. See [Roadmap](#roadmap) for planned features.

## Modules

1. **Library** (`/lib`)
   - Core TypeScript analysis engine
   - Parallel processing with Rayon
   - OXC-based AST parsing and semantic analysis

2. **CLI** (`/cli`)
   - Command-line interface exposing library features
   - Three output formats: Console (ANSI), JSON, HTML
   - Built with clap and color-eyre

3. **TypeScript Handlers** (`/ts`)
   - TypeScript type definitions for watch event handlers
   - Designed for use with Bun runtime (planned feature)

## Features

### Type Errors in Source Code

**Command:** `ta source <filter>`

Analyzes TypeScript files for type errors using OXC's semantic analyzer.

**Options:**

- `<filter>` - Glob pattern (default: `src/**/*.ts`)
- `--filter <text>` - Filter errors by message or scope
- `--max-errors <n>` - Limit number of errors reported (default: 100)
- `--include-tests` - Include test files in analysis (⚠️ *not yet implemented*)

**Output includes:**

- `message` - Error description from OXC diagnostics
- `line`, `column` - Error location
- `scope` - Context where error occurred:
    - `${symbol}` - Inside a function
    - `${class}::${method}` - Inside a class method
    - `global` - At module/file root level
- `block` - Code snippet where error occurred

**Formatting:**

- Console: Syntax highlighting with ANSI escape codes
- HTML: Wrapped in `<div class="error-block">` with `data-error-id` attribute
- JSON: Raw structured data

### Exported Symbols

**Command:** `ta symbols <filter>`

Extracts all exported symbols from TypeScript files.

**Options:**

- `<filter>` - Glob pattern or file path
- `--exported-only` - Only show exported symbols (default: all symbols)

**Symbol types detected:**

- Functions (with parameters and type annotations)
- Classes (with properties and methods)
- Interfaces
- Type aliases
- Enums
- Variables

**Output fields:**

- `name`, `kind`, `file`
- `start_line`, `end_line`
- `exported` - Boolean flag
- `parameters` - For functions (name + type annotation)
- `properties` - For classes (name + type annotation)

### Type Tests

**Command:** `ta test <filter>`

Detects type tests in test files.

**Detection criteria:**

- Looks for `describe()` → `it()`/`test()` block structure
- Identifies `type cases = [...]` patterns
- Reports test status (Passing/Failing/NoTypeCases)

**Output:**

- `file`, `describe_block`, `test_name`, `line`
- `has_type_cases` - Boolean
- `status` - Test status enum

### File Dependencies

**Command:** `ta file <filter>`

Analyzes file-level dependencies.

**Detects:**

- `import` declarations
- `export ... from` statements
- Re-exports (`export *`)

**Output:**

- `file` - Source file path
- `repo_dependencies` - Local file imports
- `external_dependencies` - Package imports

### Symbol Dependencies

**Command:** `ta deps <filter>`

Analyzes symbol-level dependencies (planned feature).

**Scope types:**

- `local` - Symbol in same file
- `repo` - Symbol in different file (same repo)
- `module` - Symbol in different monorepo module
- `external` - Symbol from external package

⚠️ *Symbol-level dependency tracking not fully implemented*

### File Watcher

**Command:** `ta watch [paths...]`

Watches TypeScript files for changes and detects events.

**Current implementation:**

- Monitors file system changes with debouncing (500ms)
- Detects and logs events to console
- Events supported:
    - `SymbolAdded` - New exported symbol detected
    - `SymbolRemoved` - Exported symbol deleted
    - `TestStatusChanged` - Type test status changed
    - `NewFailingTest` - Previously passing test now fails
    - `TestFixed` - Previously failing test now passes
    - `NewTestAdded` - New test block added

**Planned features** (⚠️ *not yet implemented*):

- External handler execution via `--${Event} ${Executable}` syntax
- TypeScript handler support with Bun:

  ```ts
  export const onSourceFileChanged: SourceFileChangedHandler = (evt) => { ... }
  ```

- Additional events: `SourceFileChanged`, `SourceFileCreated`, `SourceFileRemoved`, `SymbolRenamed`, `ModuleDepChanged`, `ExternalDepChanged` 

## Output Formats

All commands support `--format <type>` where type is:

### Console (default)

- ANSI escape sequences for terminal colors and formatting
- Syntax highlighting for TypeScript code blocks
- Concise, human-readable output
- Data to STDOUT, progress/status to STDERR

### JSON (`--format json`)

- Structured data serialization via serde_json
- Complete information for programmatic consumption
- All fields included (spans, locations, metadata)

### HTML (`--format html`)

- Wrapped in semantic HTML elements
- Metadata in `data-*` attributes
- CSS class-based styling hooks
- Example: `<div class="error-block" data-error-id="...">`

## Installation

```bash
# Build from source
cargo build --release

# Run directly
cargo run -p cli -- source "src/**/*.ts"

# Install globally
cargo install --path cli
```

## Examples

```bash
# Analyze all source files for type errors
ta source "src/**/*.ts"

# Extract symbols with JSON output
ta --format json symbols "lib/**/*.ts" > symbols.json

# Watch current directory for changes
ta watch .

# Analyze specific file's dependencies
ta file src/analyzer.rs
```

## Roadmap

### Planned Features

- [ ] External handler execution in watch mode (Bun + native executables)
- [ ] Test file exclusion in `ta source` (via `--include-tests` flag)
- [ ] Parse actual error IDs from OXC diagnostics (currently hardcoded)
- [ ] Symbol-level dependency analysis
- [ ] Additional watch events (file changes, renames, etc.)
- [ ] Enhanced JSON output with console/HTML representations

### Completed

- [x] Type error detection with scope tracking
- [x] Symbol extraction (functions, classes, types, etc.)
- [x] File dependency analysis
- [x] Type test detection
- [x] File watching with event detection
- [x] Three output formats (Console/JSON/HTML)
- [x] Parallel file processing with Rayon

## Technical Details

- **Parser:** OXC 0.30 (Oxidation Compiler)
- **CLI:** clap 4.5 (derive API)
- **Error Handling:** color-eyre + thiserror
- **Parallelism:** Rayon for multi-threaded analysis
- **File Watching:** notify-debouncer-full
- **Edition:** Rust 2021

## Performance

OXC provides 10-50x faster parsing than TSC. The analyzer leverages:

- Arena allocation for zero-cost AST memory management
- Parallel file processing (disabled for single files)
- Debounced file watching (500ms delay)
- Release builds with LTO enabled
