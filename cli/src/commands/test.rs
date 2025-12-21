use clap::Parser;
use color_eyre::eyre::{Result, Context};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::OutputFormat;
use ta_lib::models::TestStatus;

/// Detect type tests in source files
#[derive(Parser, Debug)]
pub struct TestArgs {
    /// Glob pattern for test files
    #[arg(default_value = "src/**/*.test.ts")]
    pub pattern: String,

    /// Only show failing tests
    #[arg(short, long)]
    pub failing: bool,
}

pub fn handle_test(args: TestArgs, format: OutputFormat) -> Result<()> {
    log::debug!("Handling test command with args: {:?}", args);

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
        eprintln!("No test files found matching pattern: {}", args.pattern);
        return Ok(());
    }

    eprintln!("Scanning {} files for tests...", files.len());
    let result = analyzer.analyze_files(&files)?;

    let mut tests = result.tests;

    if args.failing {
        tests.retain(|t| t.status == TestStatus::Failing);
    }

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&tests).unwrap());
        }
        _ => {
            for test in &tests {
                println!(
                    "[{:?}] {} > {} ({}:{})",
                    test.status, test.describe_block, test.test_name, test.file, test.line
                );
            }
        }
    }

    eprintln!("Found {} tests.", tests.len());

    Ok(())
}