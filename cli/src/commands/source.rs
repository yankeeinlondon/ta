use clap::Parser;
use color_eyre::eyre::{Result, Context, eyre};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::{OutputFormatter, OutputFormat};
use ignore::WalkBuilder;
use colored::*;

/// Expand brace patterns like {a,b,c} into multiple patterns
/// Example: "{src,scripts}/**/*.{ts,tsx}" -> ["./src/**/*.ts", "./src/**/*.tsx", "./scripts/**/*.ts", "./scripts/**/*.tsx"]
fn expand_braces(pattern: &str) -> Vec<String> {
    let mut result = vec![pattern.to_string()];

    // Keep expanding until no more braces found
    loop {
        let mut new_result = Vec::new();
        let mut changed = false;

        for p in &result {
            if let Some(start) = p.find('{') {
                if let Some(end) = p[start..].find('}') {
                    let end = start + end;
                    let prefix = &p[..start];
                    let suffix = &p[end + 1..];
                    let options = &p[start + 1..end];

                    // Split by comma and expand
                    for option in options.split(',') {
                        new_result.push(format!("{}{}{}", prefix, option, suffix));
                    }
                    changed = true;
                } else {
                    new_result.push(p.clone());
                }
            } else {
                new_result.push(p.clone());
            }
        }

        result = new_result;
        if !changed {
            break;
        }
    }

    // Normalize patterns to start with ./ to match WalkBuilder output
    result.into_iter().map(|p| {
        if p.starts_with("./") {
            p
        } else {
            format!("./{}", p)
        }
    }).collect()
}

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

    /// Custom glob pattern (default: {src,scripts}/**/*.{ts,tsx})
    #[arg(long)]
    pub glob: Option<String>,
}

pub fn handle_source(args: SourceArgs, format: OutputFormat, verbose: bool) -> Result<()> {
    log::debug!("Handling source command with args: {:?}", args);

    let options = AnalysisOptions {
        parallel: true,
        ..Default::default()
    };

    let analyzer = Analyzer::new(options);

    // Determine glob pattern
    let default_glob = if args.include_tests {
        "{src,scripts}/**/*.{ts,tsx}"
    } else {
        "{src,scripts}/**/*.{ts,tsx}"
    };
    let glob_pattern = args.glob.as_deref().unwrap_or(default_glob);

    // Show glob pattern in verbose mode
    if verbose {
        eprintln!("Using glob pattern: {}", glob_pattern.cyan());
    }

    let mut files = Vec::new();

    // Use custom glob pattern if provided, otherwise use default logic
    if args.glob.is_some() {
        // Manually expand brace patterns since globset doesn't support them
        let expanded_patterns = expand_braces(glob_pattern);

        use globset::GlobSetBuilder;
        let mut builder = GlobSetBuilder::new();
        for pattern in &expanded_patterns {
            builder.add(globset::Glob::new(pattern).wrap_err("Invalid glob pattern")?);
        }
        let glob_set = builder.build().wrap_err("Failed to build glob set")?;

        log::debug!("Expanded patterns: {:?}", expanded_patterns);

        // Walk all files in current directory
        let walker = WalkBuilder::new(".")
            .standard_filters(false)  // Don't use standard filters when using custom glob
            .git_ignore(true)         // But still respect .gitignore
            .git_exclude(true)        // Respect .git/info/exclude
            .filter_entry(|e| {
                // Never descend into .git directory
                e.file_name() != ".git"
            })
            .build();

        for entry in walker {
            let entry = entry.wrap_err("Failed to walk directory")?;

            if let Some(file_type) = entry.file_type() {
                if !file_type.is_file() {
                    continue;
                }
            }

            let path = entry.path();
            log::debug!("Testing path: {:?} against glob", path);
            if glob_set.is_match(path) {
                log::debug!("  ✓ Matched!");
                files.push(path.to_path_buf());
            }
        }
    } else {
        // Use ignore crate to walk files, respecting .gitignore
        // BASE pattern: all TypeScript source files in src/ and scripts/ directories
        let walker = WalkBuilder::new(".")
            .standard_filters(true)  // Respects .gitignore, .ignore, etc.
            .build();

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

    eprintln!("Analyzing {} files...", files.len());
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
