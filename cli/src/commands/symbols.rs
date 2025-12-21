use clap::Parser;
use color_eyre::eyre::{Result, Context};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::{OutputFormatter, OutputFormat};

/// Extract symbols from source files
#[derive(Parser, Debug)]
pub struct SymbolsArgs {
    /// Glob pattern or file path to analyze
    #[arg(default_value = "src/**/*.ts")]
    pub pattern: String,

    /// Only show exported symbols
    #[arg(short, long)]
    pub exported_only: bool,

    /// Filter by symbol name
    #[arg(short, long)]
    pub name: Option<String>,
}

pub fn handle_symbols(args: SymbolsArgs, format: OutputFormat) -> Result<()> {
    log::debug!("Handling symbols command with args: {:?}", args);

    let options = AnalysisOptions {
        parallel: true,
        exported_only: args.exported_only,
        ..Default::default()
    };

    let analyzer = Analyzer::new(options);
    
    let mut files = Vec::new();
    for entry in glob::glob(&args.pattern).wrap_err("Failed to read glob pattern")? {
        let path = entry.wrap_err("Invalid glob entry")?;
        if path.is_file() {
            files.push(path);
        }
    }

    if files.is_empty() {
        eprintln!("No files found matching pattern: {}", args.pattern);
        return Ok(());
    }

    eprintln!("Extracting symbols from {} files...", files.len());
    let result = analyzer.analyze_files(&files)?;

    let mut symbols = result.symbols;

    if let Some(name_filter) = args.name {
        symbols.retain(|s| s.name.contains(&name_filter));
    }

    let output = OutputFormatter::format_symbols(&symbols, format);
    println!("{}", output);

    eprintln!("Found {} symbols.", symbols.len());

    Ok(())
}
