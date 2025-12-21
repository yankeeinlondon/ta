---
name: cli
description: Expert knowledge for building Command Line Interfaces (CLIs) in Rust and TypeScript, covering argument parsing, terminal styling, escape codes, interactive prompts, and design patterns that follow Unix philosophy
last_updated: 2025-12-20T10:30:00Z
hash: fe8c69e954d33368
---

# CLI Development

Expert guidance for building professional Command Line Interfaces in Rust and TypeScript, from argument parsing to terminal control and design patterns.

## Core Principles

- **Follow the Unix Philosophy**: Do one thing well, use standard streams (stdout for data, stderr for logs), support piping and composition
- **Predictability Over Magic**: Follow POSIX conventions for flags (`-f` short, `--force` long), use consistent command structures
- **Detect the Environment**: Check for TTY before using colors/animations, respect `NO_COLOR` and `COLORTERM` environment variables
- **Provide Feedback**: Never leave users with a blinking cursor during long operations (use spinners or progress bars)
- **Make It Discoverable**: Support `-h`/`--help`, provide helpful error messages with suggestions ("Did you mean X?")
- **Choose the Right Tool**: Rust for performance and single-binary distribution, TypeScript for rapid prototyping and npm ecosystem integration
- **Separate Data from Logs**: Output data on stdout, logs/errors on stderr to enable clean piping (`tool | jq .`)
- **Handle Interruption Gracefully**: Implement proper signal handling for Ctrl+C and cleanup
- **Be Idempotent**: Running the same command twice should not error if the desired state is already met
- **Offer Machine-Readable Output**: Provide `--json` or `--format` flags to make your CLI scriptable

## Quick Reference

### Rust CLI Stack

```rust
// Cargo.toml dependencies
[dependencies]
clap = { version = "4", features = ["derive"] }  // Arg parsing
colored = "2"                                     // Colors (simple)
indicatif = "0.17"                               // Progress bars
dialoguer = "0.11"                               // Interactive prompts
anyhow = "1"                                     // Error handling
crossterm = "0.27"                               // Terminal control

// Basic structure
use clap::Parser;

#[derive(Parser)]
#[command(name = "mytool", version, about)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { file: String },
}
```

### TypeScript CLI Stack

```typescript
// package.json dependencies
{
  "dependencies": {
    "commander": "^11.0.0",    // Arg parsing
    "chalk": "^5.0.0",         // Colors
    "clack": "^0.7.0",         // Interactive prompts
    "zod": "^3.22.0"           // Validation
  }
}

// Basic structure
import { Command } from 'commander';
import chalk from 'chalk';

const program = new Command();

program
  .name('mytool')
  .version('1.0.0')
  .description('My awesome CLI');

program
  .command('run <file>')
  .option('-v, --verbose', 'verbose output')
  .action((file, options) => {
    console.log(chalk.green('Running'), file);
  });

program.parse();
```

## Topics

### Libraries & Tools

- [Libraries](./libraries.md) - Comprehensive comparison of Rust and TypeScript CLI libraries
- [Styling & Colors](./styling.md) - Console colors, escape codes, terminal control, and visual feedback

### Design & Patterns

- [Design Patterns](./design-patterns.md) - Command structure, POSIX conventions, interaction patterns, and Unix philosophy

## Common Patterns

### Progress Feedback

```rust
// Rust with indicatif
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(100);
pb.set_style(ProgressStyle::default_bar()
    .template("[{elapsed_precise}] {bar:40} {pos}/{len} {msg}")
    .progress_chars("=>-"));

for i in 0..100 {
    pb.inc(1);
    // Do work...
}
pb.finish_with_message("Done!");
```

```typescript
// TypeScript with clack
import * as p from '@clack/prompts';

const s = p.spinner();
s.start('Fetching data');
await doWork();
s.stop('Data fetched');
```

### Interactive Prompts

```rust
// Rust with dialoguer
use dialoguer::{Select, Confirm};

let selection = Select::new()
    .with_prompt("Choose environment")
    .items(&["dev", "staging", "prod"])
    .interact()?;

let confirmed = Confirm::new()
    .with_prompt("Continue?")
    .interact()?;
```

```typescript
// TypeScript with clack
import * as p from '@clack/prompts';

const env = await p.select({
  message: 'Choose environment',
  options: [
    { value: 'dev', label: 'Development' },
    { value: 'staging', label: 'Staging' },
    { value: 'prod', label: 'Production' },
  ],
});

const confirmed = await p.confirm({
  message: 'Continue?',
});
```

### TTY Detection & Color Control

```rust
// Rust - detect TTY
use std::io::IsTerminal;

if std::io::stdout().is_terminal() {
    println!("{}", "Colorful output".green());
} else {
    println!("Plain output");
}
```

```typescript
// TypeScript - detect TTY
import chalk from 'chalk';

const isInteractive = process.stdout.isTTY;

if (isInteractive) {
  console.log(chalk.green('Colorful output'));
} else {
  console.log('Plain output');
}
```

## Resources

- [CLIG Guidelines](https://clig.dev/) - Command Line Interface Guidelines
- [POSIX Utility Conventions](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html)
- [XTerm Control Sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)
- [ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code)
