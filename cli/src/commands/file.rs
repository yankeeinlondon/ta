use clap::Parser;
use color_eyre::eyre::{Result, Context, eyre};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::OutputFormat;
use ignore::WalkBuilder;
use colored::Colorize;

/// Analyze file-level dependencies (imports/exports) for all source files
#[derive(Parser, Debug)]
pub struct FileArgs {
    /// Optional filter(s) to match against source file paths (OR'd together)
    #[arg(value_name = "FILTER")]
    pub filters: Vec<String>,
}

pub fn handle_file(args: FileArgs, format: OutputFormat) -> Result<()> {
    log::debug!("Handling file command with args: {:?}", args);

    let options = AnalysisOptions {
        parallel: true,
        ..Default::default()
    };

    let analyzer = Analyzer::new(options);

    // Use ignore crate to walk files, respecting .gitignore
    // BASE pattern: same as source command - all TypeScript source files
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

        // Exclude test files
        if path_str.ends_with(".test.ts") ||
           path_str.ends_with(".spec.ts") ||
           path_str.ends_with(".test.tsx") ||
           path_str.ends_with(".spec.tsx") {
            continue;
        }

        files.push(path.to_path_buf());
    }

    // Apply user filters if provided (OR'd together)
    if !args.filters.is_empty() {
        files.retain(|f| {
            let path_str = f.to_string_lossy();
            args.filters.iter().any(|filter| path_str.contains(filter.as_str()))
        });
    }

    if files.is_empty() {
        return Err(eyre!("No source files found"));
    }

    eprintln!("Analyzing dependencies for {} files...", files.len());
    let result = analyzer.analyze_files(&files)?;

    // Use file_imports which contains resolved information
    let file_imports = &result.file_imports;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&file_imports).unwrap());
        }
        OutputFormat::Html => {
            println!("<div class='file-dependencies'>");
            for file_import in file_imports {
                println!("  <div class='file-dep'>");
                println!("    <div class='file'>{}</div>", file_import.file);
                println!("    <ul class='imports'>");
                for import in &file_import.imports {
                    // Resolve import path
                    let resolved = ta_lib::dependencies::resolve_import_path(
                        &import.source,
                        std::path::Path::new(&file_import.file)
                    );
                    let display_path = if let Some(resolved_path) = resolved {
                        resolved_path.to_string_lossy().to_string()
                    } else {
                        format!("{} (external)", import.source)
                    };
                    println!("      <li>{}</li>", display_path);
                }
                println!("    </ul>");
                println!("  </div>");
            }
            println!("</div>");
        }
        OutputFormat::Console => {
            if file_imports.is_empty() {
                println!("No dependencies found.");
            } else {
                for file_import in file_imports {
                    println!("{}:", file_import.file.blue());
                    for import in &file_import.imports {
                        // Resolve import path
                        let resolved = ta_lib::dependencies::resolve_import_path(
                            &import.source,
                            std::path::Path::new(&file_import.file)
                        );
                        let display_path = if let Some(resolved_path) = resolved {
                            resolved_path.to_string_lossy().to_string()
                        } else {
                            format!("{} (external)", import.source)
                        };
                        println!("  → {}", display_path);
                    }
                    println!();
                }
            }
        }
    }

    let total_imports: usize = file_imports.iter().map(|f| f.imports.len()).sum();
    eprintln!("Found {} files with {} total imports.", file_imports.len(), total_imports);

    Ok(())
}
