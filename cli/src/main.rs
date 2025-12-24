use clap::{Parser, Subcommand};
use color_eyre::eyre::{Result, WrapErr};
use std::path::PathBuf;
use ta_lib::output::OutputFormat;
use colored::control;

pub mod error;
pub mod commands;
pub mod utils;

use commands::source::{handle_source, SourceArgs};
use commands::symbols::{handle_symbols, SymbolsArgs};
use commands::test::{handle_test, TestArgs};
use commands::file::{handle_file, FileArgs};
use commands::deps::{handle_deps, DepsArgs};
use commands::watch::{handle_watch, WatchArgs};

#[derive(Parser)]
#[command(name = "ta")]
#[command(about = "TypeScript Analyzer - High-performance AST analysis")]
#[command(version, long_about = None)]
pub struct Cli {
    /// Change to this directory before executing command
    #[arg(short, long, global = true, value_name = "PATH")]
    pub dir: Option<PathBuf>,

    /// Output as JSON instead of console format
    #[arg(long, global = true, conflicts_with = "html")]
    pub json: bool,

    /// Output as HTML instead of console format
    #[arg(long, global = true, conflicts_with = "json")]
    pub html: bool,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Theme to use for syntax highlighting
    #[arg(long, global = true, env = "TA_THEME")]
    pub theme: Option<String>,

    /// Theme to use for light mode (overrides --theme)
    #[arg(long, global = true, env = "TA_LIGHT_THEME")]
    pub light_theme: Option<String>,

    /// Theme to use for dark mode (overrides --theme)
    #[arg(long, global = true, env = "TA_DARK_THEME")]
    pub dark_theme: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze source files for type errors
    Source(SourceArgs),
    /// Extract symbols from source files
    Symbols(SymbolsArgs),
    /// Detect type tests in source files
    Test(TestArgs),
    /// Analyze file-level dependencies (imports/exports) for source files
    File(FileArgs),
    /// Analyze module dependencies
    Deps(DepsArgs),
    /// Watch for file changes and run analysis
    Watch(WatchArgs),
    /// List available syntax highlighting themes
    ListThemes,
}

fn setup_colors() {
    // Respect CLICOLOR_FORCE to always enable colors
    if std::env::var("CLICOLOR_FORCE").is_ok() && std::env::var("CLICOLOR_FORCE").unwrap() != "0" {
        control::set_override(true);
        return;
    }

    // Respect NO_COLOR environment variable and TTY detection
    if std::env::var("NO_COLOR").is_ok() || !atty::is(atty::Stream::Stdout) {
        control::set_override(false);
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    setup_colors();

    let cli = Cli::parse();

    // Change directory BEFORE doing anything else (critical for monorepo support)
    if let Some(dir) = &cli.dir {
        std::env::set_current_dir(dir)
            .wrap_err_with(|| format!("Failed to change to directory: {}", dir.display()))?;
    }

    setup_logging(cli.verbose);

    // Derive OutputFormat from flags
    let format = if cli.json {
        OutputFormat::Json
    } else if cli.html {
        OutputFormat::Html
    } else {
        OutputFormat::Console
    };

    match cli.command {
        Commands::Source(args) => handle_source(args, format, cli.verbose)?,
        Commands::Symbols(args) => handle_symbols(args, format)?,
        Commands::Test(args) => handle_test(args, format)?,
        Commands::File(args) => handle_file(args, format)?,
        Commands::Deps(args) => handle_deps(args, format)?,
        Commands::Watch(args) => handle_watch(args, format)?,
        Commands::ListThemes => {
            let themes = ta_lib::highlighting::themes::list_available_themes();
            println!("Available themes:");
            for theme in themes {
                println!("  {}", theme);
            }
        }
    }

    Ok(())
}

fn setup_logging(_verbose: bool) {
    // Only enable debug logging when DEBUG environment variable is set
    // This prevents -v flag from triggering debug logs
    let default_level = if std::env::var("DEBUG").is_ok() {
        "debug"
    } else {
        "info"
    };

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(default_level)
    )
    .format_timestamp(None)
    .format_target(false)
    .init();
}
