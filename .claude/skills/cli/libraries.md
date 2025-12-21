# CLI Libraries

Building a Command Line Interface involves several layers: parsing arguments, styling output (colors/tables), and managing user interaction (prompts/progress bars). Both Rust and TypeScript have mature ecosystems that handle these tasks elegantly.

## Rust CLI Libraries

Rust is the gold standard for modern CLI development because it produces small, fast, single-file binaries that don't require a runtime.

### Core Libraries

| Library | Category | Description |
| --- | --- | --- |
| `clap` | Arg Parsing | The industry standard. Uses a "derive" macro that lets you define CLI arguments using a simple Rust `struct`. |
| `ratatui` | TUI / UI | A powerful library for building full Terminal User Interfaces (rich dashboards and complex layouts). |
| `indicatif` | Feedback | The go-to library for beautiful progress bars and spinners. |
| `dialoguer` | Interaction | Provides interactive prompts for user input, password entry, and multi-select menus. |
| `crossterm` | Terminal Control | A cross-platform library to clear the screen, move the cursor, and handle keyboard events. |
| `anyhow` | Errors | Simplifies error handling in CLI apps so you don't have to define custom error types for every small task. |

### When to Use Rust

- Need high performance and minimal startup time
- Require a single binary for distribution (no runtime dependencies)
- Building tools for CI/CD pipelines where speed matters
- Cross-platform compatibility is critical
- The tool will be distributed to non-technical users

## TypeScript (Node.js/Bun) CLI Libraries

TypeScript is excellent for CLIs that need to be built quickly or integrated into existing JavaScript-heavy workflows.

### Core Libraries

| Library | Category | Description |
| --- | --- | --- |
| `commander` | Arg Parsing | The most established library for defining commands, options, and help text. |
| `oclif` | Framework | A "batteries-included" framework from Salesforce designed for building large, professional-grade CLIs. |
| `inquirer` | Interaction | The classic library for interactive prompts (checklists, input, lists). |
| `chalk` | Styling | The standard for adding colors and styling to terminal text output. |
| `clack` | Interaction | A modern, high-polish alternative to Inquirer with a focus on "effortless" and beautiful UI. |
| `zod` | Validation | While not CLI-specific, frequently used with arg parsers to ensure input matches strict data types. |

### When to Use TypeScript

- Building tools for web developers
- Want to leverage the massive npm ecosystem
- Need to prototype an interactive tool very quickly
- The tool integrates with existing JavaScript/TypeScript projects
- Rapid iteration and development speed is more important than runtime performance

## Argument Parsing Patterns

### Rust (Clap)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mytool")]
#[command(about = "A CLI tool", long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the application
    Run {
        /// Input file
        #[arg(short, long)]
        file: String,

        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Test the application
    Test {
        /// Test pattern
        pattern: Option<String>,
    },
}
```

### TypeScript (Commander)

```typescript
import { Command } from 'commander';

const program = new Command();

program
  .name('mytool')
  .description('A CLI tool')
  .version('1.0.0');

program
  .command('run')
  .description('Run the application')
  .option('-f, --file <path>', 'input file')
  .option('--format <type>', 'output format', 'json')
  .action((options) => {
    console.log('Running with:', options);
  });

program
  .command('test [pattern]')
  .description('Test the application')
  .action((pattern) => {
    console.log('Testing:', pattern);
  });

program.parse();
```

## Related

- [Styling & Colors](./styling.md) - Visual feedback and terminal control
- [Design Patterns](./design-patterns.md) - CLI structure and conventions
