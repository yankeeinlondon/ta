use clap::Parser;
use color_eyre::eyre::{Result, Context, eyre};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::OutputFormat;
use ta_lib::models::TestStatus;
use ignore::WalkBuilder;

/// Detect type tests in source files
#[derive(Parser, Debug)]
pub struct TestArgs {
    /// Optional filter(s) to match against test file paths (OR'd together)
    #[arg(value_name = "FILTER")]
    pub filters: Vec<String>,

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

    // Use ignore crate to walk files, respecting .gitignore
    let walker = WalkBuilder::new(".")
        .standard_filters(true)  // Respects .gitignore, .ignore, etc.
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

        // Only include test files
        if path_str.ends_with(".test.ts") ||
           path_str.ends_with(".spec.ts") ||
           path_str.ends_with(".test.tsx") ||
           path_str.ends_with(".spec.tsx") {
            files.push(path.to_path_buf());
        }
    }

    // Apply user filters if provided (OR'd together)
    // Multiple filters: ta test foo bar → files with "foo" OR "bar" in path
    if !args.filters.is_empty() {
        files.retain(|f| {
            let path_str = f.to_string_lossy();
            // Match if ANY filter is a substring of the path
            args.filters.iter().any(|filter| path_str.contains(filter.as_str()))
        });
    }

    if files.is_empty() {
        return Err(eyre!("No test files found"));
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
