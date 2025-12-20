# CLI Design Document

## Overview

This document outlines the design for the command-line interface (CLI) of the TypeScript Analyzer (TA) project. The CLI provides a user-friendly interface to the core analysis library, supporting all major analysis features with multiple output formats and flexible filtering options.

## Architecture

### CLI Structure

The CLI uses clap's Derive API for clean, declarative command definitions. The structure follows a subcommand pattern where each analysis feature becomes a subcommand:

```
ta [GLOBAL_OPTIONS] <SUBCOMMAND> [SUBCOMMAND_OPTIONS]
```

### Global Options

```rust
#[derive(Parser)]
#[command(name = "ta")]
#[command(about = "TypeScript Analyzer - High-performance AST analysis for TypeScript codebases")]
#[command(version, long_about = None)]
pub struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "console")]
    pub format: OutputFormat,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long, default_value = ".ta.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Console output with ANSI colors (default)
    Console,
    /// Structured JSON output
    Json,
    /// HTML output with semantic markup
    Html,
}
```

### Subcommands

Each analysis feature maps to a subcommand:

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Analyze type errors in source files
    Source(SourceArgs),

    /// List exported symbols with filtering
    Symbols(SymbolsArgs),

    /// Detect and report on type tests
    Test(TestArgs),

    /// Analyze file-level dependencies
    File(FileArgs),

    /// Analyze symbol-level dependencies
    Deps(DepsArgs),

    /// Watch for file changes and trigger handlers
    Watch(WatchArgs),
}
```

## Command Specifications

### Source Command (`ta source <filter>`)

Analyzes type errors in source files with rich context information.

```rust
#[derive(Parser)]
pub struct SourceArgs {
    /// File or directory filter (supports glob patterns)
    #[arg(value_name = "FILTER")]
    pub filter: String,

    /// Include test files in analysis (normally excluded)
    #[arg(long)]
    pub include_tests: bool,

    /// Maximum number of errors to report
    #[arg(long, default_value = "100")]
    pub max_errors: usize,

    /// Focus on specific error types
    #[arg(long, value_delimiter = ',')]
    pub error_types: Option<Vec<String>>,

    /// Show only errors in specific scopes
    #[arg(long)]
    pub scope: Option<String>,
}
```

**Usage Examples:**
```bash
# Analyze all TypeScript files in current directory
ta source "**/*.ts"

# Analyze specific file with JSON output
ta source src/main.ts --format json

# Analyze with custom error limits
ta source "src/**/*.ts" --max-errors 50

# Focus on specific error types
ta source . --error-types "TS2322,TS2339"

# HTML output for web viewing
ta source . --format html > analysis.html
```

### Symbols Command (`ta symbols <filter>`)

Lists exported symbols with filtering capabilities.

```rust
#[derive(Parser)]
pub struct SymbolsArgs {
    /// File or directory filter
    #[arg(value_name = "FILTER")]
    pub filter: String,

    /// Filter by symbol type
    #[arg(short, long, value_enum)]
    pub kind: Option<SymbolKind>,

    /// Filter by symbol name pattern (regex)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Show only symbols from specific file
    #[arg(long)]
    pub file: Option<String>,

    /// Include private/non-exported symbols
    #[arg(long)]
    pub all: bool,

    /// Sort symbols by name, file, or kind
    #[arg(long, value_enum, default_value = "name")]
    pub sort: SortOrder,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Type,
    Variable,
    Enum,
    All,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SortOrder {
    Name,
    File,
    Kind,
}
```

**Usage Examples:**
```bash
# List all exported functions
ta symbols . --kind function

# Find symbols matching pattern
ta symbols "**/*.ts" --name ".*Handler.*"

# List all symbols from specific file
ta symbols . --file src/api.ts

# Include private symbols
ta symbols . --all

# Sort by file location
ta symbols . --sort file
```

### Test Command (`ta test`)

Detects and reports on type tests in test files.

```rust
#[derive(Parser)]
pub struct TestArgs {
    /// Custom test directory path
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// Test framework to detect (auto-detect if not specified)
    #[arg(long, value_enum)]
    pub framework: Option<TestFramework>,

    /// Show only failing tests
    #[arg(long)]
    pub failing: bool,

    /// Show only tests without type cases
    #[arg(long)]
    pub missing: bool,

    /// Include test execution status
    #[arg(long)]
    pub status: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum TestFramework {
    Jest,
    Vitest,
    Mocha,
    Ava,
    Auto,
}
```

**Usage Examples:**
```bash
# Analyze tests in default location
ta test

# Analyze custom test directory
ta test --dir tests/unit

# Show only failing type tests
ta test --failing

# Check for missing type cases
ta test --missing

# Include test execution status
ta test --status
```

### File Dependencies Command (`ta file <filter>`)

Analyzes file-level dependencies.

```rust
#[derive(Parser)]
pub struct FileArgs {
    /// File or directory filter
    #[arg(value_name = "FILTER")]
    pub filter: String,

    /// Dependency scope to analyze
    #[arg(short, long, value_enum, default_value = "all")]
    pub scope: DependencyScope,

    /// Maximum dependency depth
    #[arg(long)]
    pub depth: Option<usize>,

    /// Show dependency graph in DOT format
    #[arg(long)]
    pub graph: bool,

    /// Exclude external packages
    #[arg(long)]
    pub local_only: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DependencyScope {
    All,
    Repo,
    External,
}
```

**Usage Examples:**
```bash
# Show all dependencies for files
ta file "**/*.ts"

# Show only local repo dependencies
ta file . --scope repo

# Generate dependency graph
ta file . --graph > deps.dot

# Limit analysis depth
ta file . --depth 3
```

### Symbol Dependencies Command (`ta deps <filter>`)

Analyzes symbol-level dependencies with scope classification.

```rust
#[derive(Parser)]
pub struct DepsArgs {
    /// File or directory filter
    #[arg(value_name = "FILTER")]
    pub filter: String,

    /// Symbol name to analyze
    #[arg(short, long)]
    pub symbol: Option<String>,

    /// Dependency scope filter
    #[arg(short, long, value_enum)]
    pub scope: Option<SymbolDependencyScope>,

    /// Show reverse dependencies (what depends on this symbol)
    #[arg(long)]
    pub reverse: bool,

    /// Include dependency chains
    #[arg(long)]
    pub chains: bool,

    /// Maximum chain depth
    #[arg(long, default_value = "5")]
    pub max_depth: usize,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SymbolDependencyScope {
    Local,
    Repo,
    Module,
    External,
}
```

**Usage Examples:**
```bash
# Analyze dependencies for specific symbol
ta deps . --symbol "UserService"

# Show what depends on a symbol
ta deps . --symbol "Database" --reverse

# Show only external dependencies
ta deps . --scope external

# Include dependency chains
ta deps . --chains --max-depth 10
```

### Watch Command (`ta watch <handler> <...handler>`)

Watches for file changes and triggers handlers.

```rust
#[derive(Parser)]
pub struct WatchArgs {
    /// Handler specifications (--event executable)
    #[arg(value_name = "HANDLER")]
    pub handlers: Vec<String>,

    /// Watch directory (defaults to current)
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// File patterns to watch
    #[arg(short, long, value_delimiter = ',')]
    pub patterns: Option<Vec<String>>,

    /// Ignore patterns
    #[arg(long, value_delimiter = ',')]
    pub ignore: Option<Vec<String>>,

    /// Debounce delay in milliseconds
    #[arg(long, default_value = "500")]
    pub debounce: u64,

    /// Clear screen between updates
    #[arg(long)]
    pub clear: bool,
}
```

**Usage Examples:**
```bash
# Watch with TypeScript handler
ta watch --sourceFileChanged ./handlers/on-change.ts

# Watch with native executable
ta watch --symbolAdded ./scripts/on-symbol-added.sh

# Multiple handlers
ta watch --sourceFileChanged ./handlers/source.ts --testStatusChanged ./handlers/test.ts

# Custom watch patterns
ta watch --patterns "**/*.ts,**/*.tsx" --sourceFileChanged ./handler.ts
```

## Output System

### Output Format Handling

The CLI delegates output formatting to the library's `OutputFormatter`:

```rust
pub fn run_command(cli: Cli) -> Result<(), CliError> {
    let analyzer = Analyzer::new(cli.config)?;
    let result = match cli.command {
        Commands::Source(args) => {
            let analysis = analyzer.analyze_source(&args)?;
            formatter.format_type_errors(&analysis.errors, cli.format)
        }
        Commands::Symbols(args) => {
            let symbols = analyzer.extract_symbols(&args)?;
            formatter.format_symbols(&symbols, cli.format)
        }
        // ... other commands
    };

    match cli.format {
        OutputFormat::Console => println!("{}", result),
        OutputFormat::Json => println!("{}", result),
        OutputFormat::Html => println!("{}", result),
    }

    Ok(())
}
```

### Error Handling

CLI errors are categorized and presented appropriately:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Analysis error: {0}")]
    Analysis(#[from] AnalysisError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
}
```

## Configuration System

### Configuration File Support

The CLI supports TOML configuration files for default settings:

```toml
# .ta.toml
[defaults]
format = "console"
verbose = false
max_errors = 100

[source]
include_tests = false
error_types = ["TS2322", "TS2339"]

[watch]
patterns = ["**/*.ts", "**/*.tsx"]
ignore = ["node_modules/**", "**/*.test.ts"]
debounce = 500
```

### Environment Variables

Environment variables can override configuration:

```bash
TA_FORMAT=json ta source .
TA_VERBOSE=1 ta symbols .
TA_CONFIG=custom.toml ta test
```

## Advanced Features

### Shell Completions

Generate shell completion scripts using `clap_complete`:

```rust
// In build.rs
use clap_complete::{generate_to, shells::*};

fn main() {
    let mut cmd = Cli::command();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    generate_to(Bash, &mut cmd, "ta", &out_dir).unwrap();
    generate_to(Zsh, &mut cmd, "ta", &out_dir).unwrap();
    generate_to(Fish, &mut cmd, "ta", &out_dir).unwrap();
}
```

### Custom Value Parsers

Complex argument validation using custom parsers:

```rust
fn parse_filter(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("Filter cannot be empty".to_string());
    }

    // Validate glob pattern
    match glob::Pattern::new(s) {
        Ok(_) => Ok(s.to_string()),
        Err(e) => Err(format!("Invalid glob pattern: {}", e)),
    }
}

#[arg(value_parser = parse_filter)]
pub filter: String,
```

### Progress Reporting

For long-running operations, show progress:

```rust
use indicatif::{ProgressBar, ProgressStyle};

fn analyze_with_progress(files: &[PathBuf]) -> Result<AnalysisResult, CliError> {
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));

    let results: Vec<_> = files.iter().map(|file| {
        let result = analyze_file(file)?;
        pb.inc(1);
        Ok(result)
    }).collect();

    pb.finish_with_message("Analysis complete");
    // ...
}
```

## Integration with Library

### Library Interface

The CLI acts as a thin wrapper around the analysis library:

```rust
pub struct CliRunner {
    analyzer: Analyzer,
    formatter: OutputFormatter,
}

impl CliRunner {
    pub fn new(config_path: &Path) -> Result<Self, CliError> {
        let config = Config::load(config_path)?;
        let analyzer = Analyzer::new(config)?;
        let formatter = OutputFormatter::new();

        Ok(Self { analyzer, formatter })
    }

    pub fn run_source_analysis(&self, args: &SourceArgs, format: OutputFormat) -> Result<String, CliError> {
        let files = self.find_files(&args.filter)?;
        let analysis = self.analyzer.analyze_files_parallel(&files)?;
        Ok(self.formatter.format_type_errors(&analysis.errors, format))
    }

    // ... other command implementations
}
```

### Error Propagation

CLI errors are converted to user-friendly messages:

```rust
impl From<AnalysisError> for CliError {
    fn from(err: AnalysisError) -> Self {
        match err {
            AnalysisError::ParseError(msg) => {
                CliError::Analysis(format!("Failed to parse TypeScript: {}", msg))
            }
            AnalysisError::SemanticError(msg) => {
                CliError::Analysis(format!("Semantic analysis failed: {}", msg))
            }
            // ... other mappings
        }
    }
}
```

## Testing Strategy

### Unit Tests

Test individual CLI components:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parses_source_command() {
        let args = ["ta", "source", "**/*.ts", "--format", "json"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Source(source_args) => {
                assert_eq!(source_args.filter, "**/*.ts");
                assert_eq!(cli.format, OutputFormat::Json);
            }
            _ => panic!("Expected Source command"),
        }
    }

    #[test]
    fn cli_validates_required_args() {
        let args = ["ta", "source"]; // Missing filter
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn verify_cli_structure() {
        let cmd = Cli::command();
        cmd.debug_assert();
    }
}
```

### Integration Tests

Test end-to-end CLI behavior:

```rust
#[cfg(test)]
mod integration_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_source_command_basic() {
        let mut cmd = Command::cargo_bin("ta").unwrap();
        cmd.arg("source").arg("tests/fixtures/simple.ts");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("No type errors found"));
    }

    #[test]
    fn test_json_output_format() {
        let mut cmd = Command::cargo_bin("ta").unwrap();
        cmd.arg("source")
           .arg("tests/fixtures/with-errors.ts")
           .arg("--format")
           .arg("json");

        cmd.assert()
            .success()
            .stdout(predicate::str::is_json());
    }
}
```

## Performance Considerations

### Parallel Execution

Long-running commands use parallel processing:

```rust
pub fn run_parallel_analysis(&self, files: &[PathBuf]) -> Result<String, CliError> {
    if files.len() > 10 {  // Threshold for parallel processing
        let results: Vec<_> = files.par_iter()
            .map(|file| self.analyzer.analyze_single_file(file))
            .collect::<Result<_, _>>()?;

        // Merge results
        let merged = self.merge_results(results)?;
        Ok(self.formatter.format(&merged, self.format))
    } else {
        // Sequential for small file sets
        let result = self.analyzer.analyze_files(files)?;
        Ok(self.formatter.format(&result, self.format))
    }
}
```

### Memory Management

For large codebases, stream output to avoid memory issues:

```rust
pub fn stream_output<W: Write>(&self, writer: W, format: OutputFormat) -> Result<(), CliError> {
    match format {
        OutputFormat::Json => {
            serde_json::to_writer(writer, &self.result)?;
        }
        OutputFormat::Html => {
            // Stream HTML generation
            self.write_html_header(writer)?;
            for item in &self.result.items {
                self.write_html_item(writer, item)?;
            }
            self.write_html_footer(writer)?;
        }
        OutputFormat::Console => {
            // Console output is typically small enough to buffer
            writeln!(writer, "{}", self.formatted_console)?;
        }
    }
    Ok(())
}
```

## Distribution and Packaging

### Binary Distribution

Cross-platform binaries with shell completions:

```toml
# In Cargo.toml
[package]
name = "ta"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "ta"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
# ... other deps
```

### Installation Methods

- Cargo install: `cargo install ta`
- Pre-built binaries for major platforms
- Package managers (brew, apt, etc.)

### Shell Integration

Auto-install completions on first run:

```rust
fn install_completions_if_needed() -> Result<(), CliError> {
    let shell = detect_shell()?;
    let completions_path = get_completions_path(&shell)?;

    if !completions_path.exists() {
        generate_completions(&shell, &completions_path)?;
        println!("Installed shell completions. Restart your shell or run: source {}", completions_path.display());
    }

    Ok(())
}
```

This design provides a comprehensive, user-friendly CLI that exposes all the analysis capabilities of the TypeScript Analyzer library with flexible output options and excellent developer experience.