use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use ta_lib::output::OutputFormat;

pub mod error;
pub mod commands;

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
    /// Output format
    #[arg(short, long, value_enum, default_value = "console")]
    pub format: OutputFormat,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

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
    /// Get detailed information about a specific file
    File(FileArgs),
    /// Analyze module dependencies
    Deps(DepsArgs),
    /// Watch for file changes and run analysis
    Watch(WatchArgs),
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    setup_logging(cli.verbose);

    match cli.command {
        Commands::Source(args) => handle_source(args, cli.format)?,
        Commands::Symbols(args) => handle_symbols(args, cli.format)?,
        Commands::Test(args) => handle_test(args, cli.format)?,
        Commands::File(args) => handle_file(args, cli.format)?,
        Commands::Deps(args) => handle_deps(args, cli.format)?,
        Commands::Watch(args) => handle_watch(args, cli.format)?,
    }

    Ok(())
}

fn setup_logging(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(log_level)
    )
    .format_timestamp(None)
    .format_target(false)
    .init();
}
