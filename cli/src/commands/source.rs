use clap::Parser;
use color_eyre::eyre::{Result, Context, eyre};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::{OutputFormatter, OutputFormat};
use ignore::WalkBuilder;
use colored::*;

/// Analyze source files for type errors
#[derive(Parser, Debug)]
pub struct SourceArgs {
    /// Optional filter(s) to match against source file paths (OR'd together)
    #[arg(value_name = "FILTER")]
    pub filters: Vec<String>,

    /// Filter errors by message or scope
    #[arg(short, long)]
    pub error_filter: Option<String>,

    /// Include test files in analysis
    #[arg(long)]
    pub include_tests: bool,

    /// Maximum number of errors to report
    #[arg(long, default_value = "100")]
    pub max_errors: usize,
}

pub fn handle_source(args: SourceArgs, format: OutputFormat, verbose: bool) -> Result<()> {
    log::debug!("Handling source command with args: {:?}", args);

    let options = AnalysisOptions {
        parallel: true,
        ..Default::default()
    };

    let analyzer = Analyzer::new(options);

    // Use ignore crate to walk files, respecting .gitignore
    // BASE pattern: all TypeScript source files in src/ and scripts/ directories
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

        // BASE pattern: TypeScript files in src/ or scripts/ directories
        let is_in_source_dir = path_str.contains("/src/") ||
                                path_str.contains("/scripts/") ||
                                path_str.starts_with("src/") ||
                                path_str.starts_with("scripts/");

        let is_typescript = path_str.ends_with(".ts") || path_str.ends_with(".tsx");

        if !is_in_source_dir || !is_typescript {
            continue;
        }

        // Filter out test files unless --include-tests is set
        if !args.include_tests {
            // Use .ends_with() to avoid false positives like "contest.ts"
            if path_str.ends_with(".test.ts") ||
               path_str.ends_with(".spec.ts") ||
               path_str.ends_with(".test.tsx") ||
               path_str.ends_with(".spec.tsx") {
                continue;
            }
        }

        files.push(path.to_path_buf());
    }

    // Apply user filters if provided (OR'd together)
    // Multiple filters: ta source foo bar → files with "foo" OR "bar" in path
    if !args.filters.is_empty() {
        files.retain(|f| {
            let path_str = f.to_string_lossy();
            // Match if ANY filter is a substring of the path
            args.filters.iter().any(|filter| path_str.contains(filter.as_str()))
        });
    }

    if files.is_empty() {
        return Err(eyre!("No source files found"));
    }

    eprintln!("Analyzing {} files...\n", files.len());
    let result = analyzer.analyze_files(&files)?;

    let mut type_errors = result.type_errors;

    // Apply error filters with negative filter support (for filtering errors, not files)
    if let Some(filter) = args.error_filter {
        if let Some(negative_filter) = filter.strip_prefix('!') {
            // Negative filter: exclude errors matching
            type_errors.retain(|e| {
                !e.message.contains(negative_filter) && !e.scope.contains(negative_filter)
            });
        } else {
            // Positive filter: include errors matching
            type_errors.retain(|e| {
                e.message.contains(&filter) || e.scope.contains(&filter)
            });
        }
    }

    // Limit errors
    if type_errors.len() > args.max_errors {
        type_errors.truncate(args.max_errors);
    }

    let output = OutputFormatter::format_type_errors(&type_errors, format);
    println!("{}", output);

    // Calculate file statistics
    if !type_errors.is_empty() {
        // Count unique files with errors
        let mut files_with_errors = std::collections::HashSet::new();
        for error in &type_errors {
            files_with_errors.insert(&error.file);
        }
        let files_with_errors_count = files_with_errors.len();
        let files_without_errors_count = files.len().saturating_sub(files_with_errors_count);

        // Show individual success messages for files without errors when verbose
        if verbose && files_without_errors_count > 0 {
            eprintln!();
            for file_path in &files {
                let file_str = file_path.to_string_lossy().to_string();
                if !files_with_errors.contains(&file_str) {
                    eprintln!("- ✅ {} has no type errors", file_str.green());
                }
            }
            eprintln!();
        }

        // Format error count in red/bold, files-without-errors in dim/italic
        let error_count = format!("{}", type_errors.len()).red().bold();
        let without_errors_msg = format!(
            "{} file{} without errors",
            files_without_errors_count,
            if files_without_errors_count == 1 { "" } else { "s" }
        ).dimmed().italic();

        eprintln!(
            "Found {} type error{} in {} file{} ({}).",
            error_count,
            if type_errors.len() == 1 { "" } else { "s" },
            files_with_errors_count,
            if files_with_errors_count == 1 { "" } else { "s" },
            without_errors_msg
        );

        // Return exit code 1 when type errors are found (per CLI best practices)
        std::process::exit(1);
    } else {
        // Show individual success messages when verbose
        if verbose {
            eprintln!();
            for file_path in &files {
                eprintln!("- ✅ {} has no type errors", file_path.to_string_lossy().green());
            }
            eprintln!();
        }

        let file_count = format!("{}", files.len()).bold();
        let preposition = if files.len() == 1 { "in" } else { "across" };
        eprintln!(
            "- ✅ no type errors found {} {} file{}",
            preposition,
            file_count,
            if files.len() == 1 { "" } else { "s" }
        );
    }

    Ok(())
}
