# Test Fixtures for Code Highlighting

This directory contains TypeScript test files used to verify highlighting functionality.

## Files

- **basic_error.ts** - Single type error in simple function
- **multiple_errors.ts** - 5+ errors across different scopes
- **nested_scope.ts** - Error in class method within namespace
- **long_function.ts** - 30+ line function to test truncation logic
- **unicode.ts** - Emoji, Chinese characters, RTL text

## Usage

These fixtures are used by:
- Unit tests in `lib/src/highlighting/`
- Integration tests in `cli/tests/`
- Snapshot tests for output consistency

## Expected Output

Each fixture should have corresponding expected output files:
- `{name}.expected.console` - Console output with ANSI codes
- `{name}.expected.html` - HTML output
- `{name}.expected.json` - JSON output

(These will be generated via snapshot testing with `insta`)
