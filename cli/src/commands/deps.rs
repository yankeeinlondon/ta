use clap::Parser;
use color_eyre::eyre::{Result, Context, eyre};
use ta_lib::analyzer::{Analyzer, AnalysisOptions};
use ta_lib::output::OutputFormat;
use ignore::WalkBuilder;
use colored::Colorize;

/// Analyze module dependencies
#[derive(Parser, Debug)]
pub struct DepsArgs {
    /// Optional filter(s) to match against source file paths (OR'd together)
    #[arg(value_name = "FILTER")]
    pub filters: Vec<String>,

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

    // Use ignore crate to walk files, respecting .gitignore
    // BASE pattern: same as source command - all TypeScript source files
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

    // Build mapping: file → imported symbols with sources
    let mut file_to_imports: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    for file_import in &result.file_imports {
        for import_info in &file_import.imports {
            let is_external = !import_info.source.starts_with('.');

            // Apply external_only filter
            if args.external_only && !is_external {
                continue;
            } else if !args.external_only && is_external {
                continue;
            }

            // Resolve the source file path
            let source_file = if is_external {
                import_info.source.clone()
            } else {
                let resolved = ta_lib::dependencies::resolve_import_path(
                    &import_info.source,
                    std::path::Path::new(&file_import.file)
                );
                resolved.map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| import_info.source.clone())
            };

            for symbol in &import_info.symbols {
                file_to_imports
                    .entry(file_import.file.clone())
                    .or_default()
                    .push((symbol.clone(), source_file.clone()));
            }
        }
    }

    // Build symbol dependencies: exported_symbol → [(imported_symbol, source_file)]
    #[derive(Debug)]
    struct SymbolDep<'a> {
        symbol_info: &'a ta_lib::models::SymbolInfo,
        depends_on: Vec<(String, String)>, // (symbol_name, source_file)
    }

    let mut symbol_deps = Vec::new();

    for symbol_info in &result.symbols {
        // Only consider exported symbols
        if !symbol_info.exported {
            continue;
        }

        // Get imports used in this symbol's file
        if let Some(imports) = file_to_imports.get(&symbol_info.file) {
            symbol_deps.push(SymbolDep {
                symbol_info,
                depends_on: imports.clone(),
            });
        }
    }

    match format {
        OutputFormat::Json => {
            let output: Vec<_> = symbol_deps.iter().map(|dep| {
                serde_json::json!({
                    "symbol": serde_json::to_value(dep.symbol_info).unwrap(),
                    "depends_on": dep.depends_on.iter().map(|(sym, src)| {
                        serde_json::json!({
                            "symbol": sym,
                            "from": src
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            if symbol_deps.is_empty() {
                println!("No symbol dependencies found.");
            } else {
                // Sort by symbol name
                symbol_deps.sort_by(|a, b| a.symbol_info.name.cmp(&b.symbol_info.name));

                for dep in &symbol_deps {
                    // Use colored signature display
                    let signature = ta_lib::output::OutputFormatter::format_symbol_signature_colored(dep.symbol_info);
                    let location = format!("{}:{}", dep.symbol_info.file, dep.symbol_info.start_line).blue();

                    println!("{} {}", signature, location);

                    // Show JSDoc if present
                    if let Some(jsdoc) = &dep.symbol_info.jsdoc {
                        println!("  {}", jsdoc.dimmed().italic());
                    }

                    if dep.depends_on.is_empty() {
                        println!("  (no dependencies)");
                    } else {
                        for (symbol, source) in &dep.depends_on {
                            println!("  → {} {} {}",
                                symbol,
                                "from".white().dimmed(),
                                source
                            );
                        }
                    }
                    println!();
                }

                eprintln!("Found {} exported symbols with dependencies.", symbol_deps.len());
            }
        }
    }

    Ok(())
}
