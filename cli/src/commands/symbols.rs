use clap::Parser;
use color_eyre::eyre::{Result, Context, eyre};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::{OutputFormatter, OutputFormat};
use ignore::WalkBuilder;

/// Extract symbols from source files
#[derive(Parser, Debug)]
pub struct SymbolsArgs {
    /// Optional filter(s) to match against source file paths (OR'd together)
    #[arg(value_name = "FILTER")]
    pub filters: Vec<String>,

    /// Filter symbol names (prefix with ! for negative match)
    #[arg(short = 'n', long = "name")]
    pub symbol_filter: Option<String>,

    /// Only show exported symbols
    #[arg(short, long)]
    pub exported_only: bool,
}

pub fn handle_symbols(args: SymbolsArgs, format: OutputFormat) -> Result<()> {
    log::debug!("Handling symbols command with args: {:?}", args);

    let options = AnalysisOptions {
        parallel: true,
        exported_only: args.exported_only,
        ..Default::default()
    };

    let analyzer = Analyzer::new(options);

    // Use ignore crate to walk files, respecting .gitignore
    // BASE pattern: all TypeScript source files in src/ and scripts/ directories
    let walker = WalkBuilder::new(".")
        .standard_filters(true)
        .build();

    let mut files = Vec::new();
    for entry in walker {
        let entry = entry.wrap_err("Failed to walk directory")?;

        if let Some(file_type) = entry.file_type() {
            if !file_type.is_file() {
                continue;
            }
        }

        let path = entry.path();
        let path_str = path.to_string_lossy();

        // BASE pattern: TypeScript files in src/ or scripts/ directories
        let is_in_source_dir = path_str.contains("/src/") ||
                                path_str.contains("/scripts/") ||
                                path_str.starts_with("src/") ||
                                path_str.starts_with("scripts/");

        let is_typescript = path_str.ends_with(".ts") || path_str.ends_with(".tsx");

        if !is_in_source_dir || !is_typescript {
            continue;
        }

        // Exclude test files
        if path_str.ends_with(".test.ts") ||
           path_str.ends_with(".spec.ts") ||
           path_str.ends_with(".test.tsx") ||
           path_str.ends_with(".spec.tsx") {
            continue;
        }

        files.push(path.to_path_buf());
    }

    // Apply file path filters if provided (OR'd together)
    if !args.filters.is_empty() {
        files.retain(|f| {
            let path_str = f.to_string_lossy();
            args.filters.iter().any(|filter| path_str.contains(filter.as_str()))
        });
    }

    if files.is_empty() {
        return Err(eyre!("No source files found"));
    }

    eprintln!("Extracting symbols from {} files...", files.len());
    let result = analyzer.analyze_files(&files)?;

    let mut symbols = result.symbols;

    // Apply symbol name filter with negative filter support (filters symbol NAMES, not files)
    if let Some(filter) = args.symbol_filter {
        if let Some(negative_filter) = filter.strip_prefix('!') {
            // Negative filter: exclude symbols whose names contain this string
            symbols.retain(|s| !s.name.contains(negative_filter));
        } else {
            // Positive filter: include symbols whose names contain this string
            symbols.retain(|s| s.name.contains(&filter));
        }
    }

    let output = OutputFormatter::format_symbols(&symbols, format);
    println!("{}", output);

    eprintln!("Found {} symbols.", symbols.len());

    Ok(())
}
