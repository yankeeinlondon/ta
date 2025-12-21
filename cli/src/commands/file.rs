use clap::Parser;
use std::path::PathBuf;
use color_eyre::eyre::Result;
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::{OutputFormatter, OutputFormat};

/// Get detailed information about a specific file
#[derive(Parser, Debug)]
pub struct FileArgs {
    /// Path to the file
    pub path: PathBuf,
}

pub fn handle_file(args: FileArgs, format: OutputFormat) -> Result<()> {
    log::debug!("Handling file command with args: {:?}", args);

    if !args.path.exists() {
        return Err(color_eyre::eyre::eyre!("File not found: {:?}", args.path));
    }

    let options = AnalysisOptions {
        parallel: false,
        ..Default::default()
    };

    let analyzer = Analyzer::new(options);
    let analysis = analyzer.analyze_single_file(&args.path)?;

    match format {
        OutputFormat::Json => {
            println!(
                "{{\"file\":\"{}\",\"errors\":{},\"symbols\":{}}}",
                args.path.display(),
                serde_json::to_string(&analysis.type_errors).unwrap(),
                serde_json::to_string(&analysis.symbols).unwrap()
            );
        }
        _ => {
            println!("File: {}", args.path.display());
            println!("\n--- Symbols ---");
            println!("{}", OutputFormatter::format_symbols(&analysis.symbols, format));
            println!("\n--- Type Errors ---");
            println!("{}", OutputFormatter::format_type_errors(&analysis.type_errors, format));
        }
    }

    Ok(())
}
