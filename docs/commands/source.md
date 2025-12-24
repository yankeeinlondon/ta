# `ta source` - Type Error Analysis

Analyzes TypeScript source files for type errors using OXC's semantic analyzer with context-aware code extraction and syntax highlighting.

## Synopsis

```bash
ta source [OPTIONS] [FILTERS]...
ta source -v [OPTIONS] [FILTERS]...  # Verbose mode
```

## Description

The `source` command performs deep type analysis on TypeScript files, detecting semantic errors like:
- Type mismatches
- Redeclarations
- Invalid operations
- Missing imports
- And all other errors detectable by OXC's TypeScript parser

Results are displayed with:
- **Syntax highlighting** (24-bit RGB ANSI colors)
- **Context-aware code extraction** (shows relevant function/method, not entire file)
- **Smart boundary detection** (stops at blank lines and closing braces for module-level errors)
- **Visual formatting** (indented code blocks, ❌ emoji markers)
- **Clickable file paths** (OSC8 hyperlinks for terminal emulators that support them)

## Options

### `-v, --verbose`

Show individual success messages for each file without errors.

**Without verbose:**
```
- ✅ no type errors found across 14 files
```

**With verbose:**
```
- ✅ ./src/api.ts has no type errors
- ✅ ./src/network.ts has no type errors
- ✅ ./src/types.ts has no type errors
...

- ✅ no type errors found across 14 files
```

## Arguments

### `[FILTERS]...`

Optional filter patterns to match against file paths. Multiple filters are OR'd together.

**Examples:**
```bash
ta source errors           # Files with "errors" in path
ta source user auth        # Files with "user" OR "auth" in path
ta source src/api          # Files under src/api/
```

**Default behavior (no filters):**
- Analyzes all `.ts` and `.tsx` files in `src/` and `scripts/` directories
- Respects `.gitignore`, `.ignore`, and other standard ignore files
- Excludes test files by default (unless `--include-tests` is specified)

## Output Example

```
Analyzing 14 files...

[❌] Identifier \`userId\` has already been declared
  in processUser at ./src/errors.ts:6:8

  function processUser() {
      let userId = 1;
      let userId = 2;  // Error
      return userId;
  }

Found 3 type errors in 2 files (12 files without errors).
```

**Note:** In the actual console output:
- The error count (`3` in this example) appears in **red and bold**
- The files-without-errors message (`12 files without errors`) appears _dimmed and italicized_
- File paths are **clickable links** (in terminals supporting OSC8 hyperlinks like iTerm2, WezTerm, etc.)
- When no errors are found: `- ✅ no type errors found in X file` (singular) or `- ✅ no type errors found across X files` (plural), where X is **bold**

## Debug Logging

To see internal debug logs (file walking, glob patterns, etc.), set the `DEBUG` environment variable:

```bash
DEBUG=1 ta source
```

This is separate from the `-v` flag, which controls user-facing output verbosity.

## Exit Codes

Following [CLI best practices](../cli-best-practices.md):
- **0** - No type errors found (success)
- **1** - Type errors detected (unsuccessful outcome)
- **9** - Invalid CLI arguments (misuse)
- **99** - Unexpected error

## See Also

- [Code and Markdown Highlighting](../code-and-md-highlighting.md) - Detailed highlighting implementation
- [CLI Best Practices](../cli-best-practices.md) - Exit code conventions

## Status

**Implementation:** ✅ Complete (v0.1.0)  
**Test Coverage:** 22/22 tests passing
