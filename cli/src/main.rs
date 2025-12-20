use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

pub mod error;

#[derive(Parser)]
#[command(name = "ta")]
#[command(about = "TypeScript Analyzer - High-performance AST analysis")]
#[command(version, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    // Placeholder for now
    Source,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_default_env().init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Source) => {
            println!("Source command not implemented yet");
        }
        None => {
             use clap::CommandFactory;
             Cli::command().print_help()?;
        }
    }

    Ok(())
}