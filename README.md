# TA (Typescript Analyzer)

> a speedy AST analyzer written in Rust and using OXC under the hood

## Modules

1. Library `/lib` 

    - provides the support code to analyze typescript files or whole code bases

2. CLI `/cli`

    - a CLI which exposes the features developed in the Library module

3. Typescript Handlers `/ts`

    - export Typescript types used for defining Typescript handlers for the `watch` command

## Features

- Type Errors in Source Code (cli: `ta source <filter>`)

    - is able to analyze either a file, a file subset, or an entire repo and report on the type errors which exist
        - when an entire repo is selected it will try to **exclude** the test files from it's report
    - in addition to the type error (id, message), line number, and other basics you'd expect in a error diagnostic this library provides the following:
        - `scope` - a text descriptor of where the error occurred following the format:
            - `${file}::${symbol}` when the error occurred inside a function
            - `${file}::${class}:${method}` when the error occurred inside a class method
            - `${file}::root` when the error occurred outside any block based symbol
        - `block` - a _plain text_ representation of the code block where the error occurred
            - if it's inside a function/class method then the full function/method definition
            - if it's scope is `root` based then just the line the error occurred with the preceding and following lines until an empty line is encountered
        - `blockHtml` - same as `block` but with HTML based code highlighting 
        - `blockConsole` - same as `block` but with code highlighting via escape sequences

- Exported Symbols (cli: `ta symbols <filter>`)

    - provides a list of all symbols which are exported in the repo
    - filtering by symbol type (function, type, class, etc.) available
    - reporting includes:
        - structured data such as:
            - name
            - file
            - start_line
            - end_line
            - ... and more on a per symbol type basis (e.g., `parameters` for functions, etc.)
        - an HTML representation of the symbol's summary (not details)
        - a Console representation of the symbol's summary (not details)

- Type Tests (cli: `ta test`)

    - will try to detect the tests directory or can be explicitly directed to it
    - with the tests directories identified it will report on type tests where a "type test":
        - Is bounded in a `describe()` -> `it()`/`test()` block structure common to many test runners
        - It will look for `type cases = [ test, test, test ]` definitions in the test files
        - Test blocks that do not have a type test block will be reported on 

- File Dependencies (cli: `ta file <filter>`)

    - identifies a file's dependencies on other files in the repo as well as external packages
    - when no filter is applied it will iterate over all source files (not test files) and report their 

- Symbol Dependencies (cli: `ta deps <filter>`)

    - identifies a symbol's dependencies on other symbols
    - these dependencies are scoped to:
        - `local` - dependency on a symbol in same file
        - `repo` - dependency on a symbol within the repo but in a different file
        - `module` - when in a monorepo, this would indicate a symbol's dependency on a symbol from another module in the monorepo
        - `external` - dependency is on a symbol from an external package

- Watcher mode (cli: `ta watch <handler> <...handler>`)

    - watches the file system for file changes and fires events for any provided handlers
    - events include
        - sourceFileChanged
        - sourceFileCreated
        - sourceFileRemoved
        - symbolRenamed
        - symbolAdded
        - symbolRemoved
        - moduleDepChanged
        - externalDepChanged
        - testStatusChanged - called whenever a _type test_ changes from one status to another
        - newFailingTest - called whenever a test that _was_ passing is now failing
        - testFixed - called whenever a test that _was_ failing is now passing
        - newTestAdded - called when a new test block is added to tests
    - handlers are defined by adding `--${Event} ${Executable}` to the `ta watch` call
        - the `${Executable}` is either something the system can execute natively (and execute permissions set on POSIX systems)
        - but it can also be a Typescript file with an event handler function:
            
            ```ts
            export const onSourceFileChanged: SourceFileChangedHandler = (evt) => { ... }
            ```

        - as long as the system has `Bun` installed and the appropriately named handler exists in the 

> **Note:** all CLI commands will output to STDOUT and where status, progress, or summary messages are provided these will be sent over STDERR
>
> **Note:** the output format is by default optimized for consumption in the terminal/console (using console escape sequences to provide useful colorful reports); however all commands support `--json` and `--html` switches to modify the output format. When JSON is output the information provided will be much more verbose.
>
> **Note:**
> 
> - JSON output is information rich and includes `console` and `html` properties which include the console-optimized and html-optimized summary output.
> - CONSOLE output is just the console-optimized (e.g., colorized with escape codes) summary information and is the least information rich
> - HTML output displays the HTML optimized summary information (same info as console but with span/class blocks indicating formatting) but it adds additional metadata in `data-xxx` properties in the wrapping `<span>` block.
