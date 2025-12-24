
## Return Status

- all CLI commands should return a non-zero response when an error condition is the outcome:
    - if the error is a misuse of the CLI's parameters or switches then we should return a `9`
    - if the error is based on a successful run of the CLI command with an unsuccessful outcome (e.g., "type errors found", etc.) then we should return `1`
    - if the error is unexpected or doesn't fit either of the two conditions above then return `99`
- all CLI commands should return a `0` code when executed successfully with a successful result.

## Debug Logging

- Use the `DEBUG` environment variable to enable debug logging, not the `-v` flag
- The `-v` (verbose) flag should control **user-facing output verbosity**, not internal debug logs
- Example: `DEBUG=1 ta source` shows internal debug logs, while `ta source -v` shows more detailed user-facing output
- This separation allows users to get more information without being overwhelmed by implementation details

## JSON Output

When the `--json` flag is used in our commands we should not only export the JSON payload but we should also make sure that in the `ts` module we define a **type** for the JSON payload's type.

### Example: Source Command

```ts
/** Scope type where the error occurred */
export type ScopeType = "Function" | "Method" | "TypeUtility" | "ModuleLevel";

/** A single type error */
export interface TypeError {
    /** Error code (e.g., "TS2322") or "error" fallback */
    id: string;
    /** Human-readable error message */
    message: string;
    /** Relative file path where error occurred */
    file: string;
    /** Line number (1-indexed) */
    line: number;
    /** Column number (1-indexed) */
    column: number;
    /** Scope identifier (e.g., "myFunction", "MyClass::method", "global") */
    scope: string;
    /** Legacy plain text code block (deprecated, use source_code instead) */
    block: string;
    /** Context-aware code extraction (optional) */
    source_code?: {
        /** Complete code of the enclosing scope */
        full_code: string;
        /** Truncated code snippet showing the error */
        display_code: string;
        /** Type of scope containing the error */
        scope_type: ScopeType;
        /** Name of the scope (function/method/class name) */
        scope_name: string;
    };
    /** Source span information */
    span: {
        /** Byte offset of error start */
        start: number;
        /** Byte offset of error end */
        end: number;
    };
}

/** JSON output from `ta source --json` */
export type SourceOutput = TypeError[];
```

**All command types** are defined and exported in the `ts` module (`ts/src/index.ts`) including:

- `SourceOutput` - Type errors from `ta source --json`
- `SymbolsOutput` - Symbols from `ta symbols --json`
- `FileOutput` - File dependencies from `ta file --json`
- `DepsOutput` - Symbol dependencies from `ta deps --json`
- `TestOutput` - Type tests from `ta test --json`

## STDOUT and STDERR

- all results of the command go to STDOUT, but
- all diagnostics, progress info, summary info, etc. go to STDERR


