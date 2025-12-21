use clap::Parser;
use color_eyre::eyre::{Result, Context};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::OutputFormat;
use std::collections::HashSet;

/// Analyze module dependencies
#[derive(Parser, Debug)]
pub struct DepsArgs {
    /// Glob pattern for files to analyze
    #[arg(default_value = "src/**/*.ts")]
    pub pattern: String,

    /// Only show external dependencies
    #[arg(short, long)]
    pub external_only: bool,
}

pub fn handle_deps(args: DepsArgs, format: OutputFormat) -> Result<()> {
    log::debug!("Handling deps command with args: {:?}", args);

    let options = AnalysisOptions {
        parallel: true,
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

    eprintln!("Analyzing dependencies for {} files...", files.len());
    let result = analyzer.analyze_files(&files)?;

    let mut deps = result.dependencies;
    
    if args.external_only {
        deps.retain(|d| !d.starts_with('.'));
    }

    // Deduplicate
    let unique_deps: HashSet<_> = deps.into_iter().collect();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&unique_deps).unwrap());
        }
        _ => {
            let mut sorted_deps: Vec<_> = unique_deps.into_iter().collect();
            sorted_deps.sort();
            for dep in sorted_deps {
                println!("{}", dep);
            }
        }
    }

    Ok(())
}
