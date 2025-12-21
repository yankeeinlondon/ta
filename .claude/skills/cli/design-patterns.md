# CLI Design Patterns

Designing a Command Line Interface is a unique challenge. Unlike GUIs, where users are guided by visual cues, a CLI relies on **predictability, discoverability, and composability**.

A great CLI follows the "Rule of Least Surprise." These patterns separate professional tools from frustrating scripts.

## 1. The Command Structure (The "Git" Style)

Most modern CLIs follow the **Command-Subcommand-Argument** pattern. This creates a hierarchical, readable structure that scales well as features grow.

**Pattern:** `[binary] [group] [command] [arguments] --flags`

**Example:** `docker container run --detach alpine`

| Component | Role | Example |
| --- | --- | --- |
| **Binary** | The name of the tool | `git` |
| **Command/Group** | The object or action being targeted | `remote`, `checkout` |
| **Subcommand** | Refines the action | `add` |
| **Arguments** | Positional data (required) | `origin [URL]` |
| **Flags/Options** | Modifiers (optional) | `--force`, `-v` |

### Implementation Example

```rust
// Rust with clap
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Container {
        #[command(subcommand)]
        cmd: ContainerCommands,
    },
}

#[derive(Subcommand)]
enum ContainerCommands {
    Run {
        #[arg(long)]
        detach: bool,
        image: String,
    },
}
```

## 2. The POSIX Standard & Flag Patterns

To make your tool feel "native" to terminal users, follow established conventions for flags.

### Short vs. Long Flags

- **Short flags** (`-f`) use a single dash and can be "bundled" (e.g., `tar -xvf`)
- **Long flags** (`--force`) use double dashes and are more readable in scripts

### The `--` Separator

Use `--` to signal the end of flags and the beginning of positional arguments. Vital if your argument starts with a dash:

```bash
rm -- -filename  # The -- prevents -filename from being parsed as a flag
```

### Booleans vs. Values

Flags should either be toggles or explicitly take a value:

```bash
--verbose               # Boolean toggle
--output=json           # Value with equals
--output json           # Value with space
```

### Common Flag Conventions

| Flag | Purpose |
| --- | --- |
| `-h`, `--help` | Display help information |
| `-v`, `--verbose` | Increase output verbosity |
| `-q`, `--quiet` | Suppress non-essential output |
| `-V`, `--version` | Display version information |
| `-f`, `--force` | Skip confirmations, override protections |
| `-n`, `--dry-run` | Show what would happen without doing it |

## 3. Feedback and UI Patterns

Since there's no "screen" to refresh, how you communicate state is critical.

### The Progress Pattern

For long-running tasks, never leave the user with a blinking cursor.

**Spinners:** Use for tasks of unknown duration

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new_spinner();
pb.set_message("Fetching data...");
pb.enable_steady_tick(Duration::from_millis(100));
// Do work...
pb.finish_with_message("Done!");
```

**Progress Bars:** Use for tasks with a known total

```rust
let pb = ProgressBar::new(100);
pb.set_style(ProgressStyle::default_bar()
    .template("[{elapsed_precise}] {bar:40} {pos}/{len} {msg}")
    .progress_chars("=>-"));

for i in 0..100 {
    pb.inc(1);
    // Do work...
}
pb.finish_with_message("Complete!");
```

### The Log Level Pattern

Allow users to control verbosity using `-v`, `-vv`, or `--quiet`:

```rust
#[derive(Parser)]
struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

// Usage
match cli.verbose {
    0 => log::set_max_level(log::LevelFilter::Warn),
    1 => log::set_max_level(log::LevelFilter::Info),
    2 => log::set_max_level(log::LevelFilter::Debug),
    _ => log::set_max_level(log::LevelFilter::Trace),
}
```

### The Color Pattern (ANSI)

**Standard Color Conventions:**
- **Success:** Green
- **Warnings:** Yellow/Orange
- **Errors:** Red
- **Info:** Blue/Cyan

**Rule:** Always detect if output is a TTY (a real person watching). If being piped to a file, disable colors automatically:

```rust
use std::io::IsTerminal;

let use_colors = std::io::stdout().is_terminal();
```

## 4. Interaction Patterns

While many CLIs are non-interactive for automation, "Human-First" CLIs use interactive patterns.

### Prompts

Basic yes/no confirmation:

```rust
use dialoguer::Confirm;

let confirmed = Confirm::new()
    .with_prompt("Are you sure?")
    .default(false)
    .interact()?;
```

### Selectable Lists

Using arrow keys to pick an option:

```rust
use dialoguer::Select;

let environments = &["development", "staging", "production"];
let selection = Select::new()
    .with_prompt("Choose environment")
    .items(environments)
    .default(0)
    .interact()?;
```

### Fuzzy Search

Allowing users to type a few letters to filter a long list:

```rust
use dialoguer::FuzzySelect;

let items = vec!["server-1", "server-2", "database-1"];
let selection = FuzzySelect::new()
    .with_prompt("Select server")
    .items(&items)
    .interact()?;
```

## 5. The "Unix Philosophy" Patterns

For a CLI to be powerful, it must play well with others.

### Standard Streams

- Use `stdout` for actual data
- Use `stderr` for logs/errors

This allows users to pipe data cleanly:

```bash
mytool --json | jq .
```

**Implementation:**

```rust
// Data goes to stdout
println!("{}", json_data);

// Logs go to stderr
eprintln!("Processing file...");
```

### Output Formatting

Offer a `--json` or `--format` flag to make your CLI a "data source" for other scripts:

```rust
#[derive(Parser)]
struct Cli {
    /// Output format
    #[arg(long, default_value = "text")]
    format: Format,
}

#[derive(ValueEnum, Clone)]
enum Format {
    Text,
    Json,
    Yaml,
}
```

### Idempotency

Running the same command twice should (ideally) not cause an error or duplicate resources if the desired state is already met.

**Bad:**
```bash
$ create-user alice
User created!
$ create-user alice
Error: User already exists!
```

**Good:**
```bash
$ create-user alice
User created!
$ create-user alice
User already exists, skipping.
```

## 6. Discoverability Patterns

### The Global Help

Every tool must support `-h` and `--help`:

```rust
#[derive(Parser)]
#[command(name = "mytool")]
#[command(about = "A helpful CLI tool", long_about = None)]
struct Cli {
    // ...
}
```

### Auto-suggestions

Provide helpful error messages with suggestions when commands are mistyped:

```text
Error: Unknown command 'statas'

Did you mean 'status'?
```

**Implementation (with clap):**

```rust
use clap::CommandFactory;

if let Err(e) = Cli::try_parse() {
    e.exit();
}
```

Clap provides this automatically for misspelled subcommands.

### Man Pages

For deep technical documentation accessible offline via `man [tool]`.

Generate man pages from clap definitions:

```rust
use clap_mangen::Man;

let man = Man::new(Cli::command());
let mut buffer: Vec<u8> = Default::default();
man.render(&mut buffer)?;
```

## 7. Configuration Patterns

### Configuration File Hierarchy

Search for config files in this order (later overrides earlier):

1. System-wide: `/etc/mytool/config.toml`
2. User-wide: `~/.config/mytool/config.toml`
3. Project-local: `./mytool.toml`
4. Environment variables: `MYTOOL_*`
5. Command-line flags

### Example Implementation

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    api_key: Option<String>,
    endpoint: String,
}

fn load_config() -> Config {
    // Load from files, merge with env vars, apply CLI overrides
}
```

## 8. Error Handling Patterns

### Exit Codes

Follow standard Unix exit codes:

- `0` - Success
- `1` - General errors
- `2` - Misuse of command (bad arguments)
- `126` - Command cannot execute
- `127` - Command not found
- `130` - Terminated by Ctrl+C

### Helpful Error Messages

**Bad:**
```text
Error: Invalid input
```

**Good:**
```text
Error: Invalid date format '2024-13-45'
  Expected: YYYY-MM-DD (e.g., 2024-01-15)
```

### Implementation with anyhow

```rust
use anyhow::{Context, Result};

fn process_file(path: &str) -> Result<()> {
    let contents = std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path))?;

    // Process contents...

    Ok(())
}
```

## Related

- [Libraries](./libraries.md) - CLI libraries and frameworks
- [Styling & Colors](./styling.md) - Visual feedback and terminal control
