
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

- when the `--json` flag is used in our commands we should not only export the JSON payload but we should also make sure that in the `ts` module we define a **type** for the JSON payload's type.
- for example:

    ```ts
    export type SourceScopeType = "ModuleLevel" | "Function" | "ClassMethod" | "TypeUtility";

    export type SourceFile = {
        id: "error" | "success";
        message: string;
        file: string;
        line: number;
        column: number;
        scope: string;
        source_code: {
            /** all of the code found in the file */
            full_code: string;
            /** the code block demonstrating the error */
            display_code: string;
            scope_type: ScopeType;
            scope_name: string;
        }
        span: {
            /** the starting line number */
            start: number;
            /** the ending line number */
            end: number;
        }
    }
    ```
