use clap::Parser;
use color_eyre::eyre::{Result, Context};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::{OutputFormatter, OutputFormat};

/// Analyze source files for type errors
#[derive(Parser, Debug)]
pub struct SourceArgs {
    /// Glob pattern or file path to analyze
    #[arg(default_value = "src/**/*.ts")]
    pub pattern: String,

    /// Filter errors by message or scope
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Include test files in analysis
    #[arg(long)]
    pub include_tests: bool,

    /// Maximum number of errors to report
    #[arg(long, default_value = "100")]
    pub max_errors: usize,
}

pub fn handle_source(args: SourceArgs, format: OutputFormat) -> Result<()> {
    log::debug!("Handling source command with args: {:?}", args);

    let options = AnalysisOptions {
        parallel: true,
        ..Default::default()
    };

    let analyzer = Analyzer::new(options);
    
    // Resolve glob pattern
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

    eprintln!("Analyzing {} files...", files.len());
    let result = analyzer.analyze_files(&files)?;

    let mut type_errors = result.type_errors;

    // Apply filters
    if let Some(filter) = args.filter {
        type_errors.retain(|e| {
            e.message.contains(&filter) || e.scope.contains(&filter)
        });
    }

    // Limit errors
    if type_errors.len() > args.max_errors {
        type_errors.truncate(args.max_errors);
    }

    let output = OutputFormatter::format_type_errors(&type_errors, format);
    println!("{}", output);

    if !type_errors.is_empty() {
        eprintln!("Found {} type errors.", type_errors.len());
    } else {
        eprintln!("No type errors found.");
    }

    Ok(())
}