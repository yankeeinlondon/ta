use std::path::{Path, PathBuf};
use rayon::prelude::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use crate::models::{TypeError, SymbolInfo, TypeTest};
use crate::{Error, Result};
use crate::type_errors::extract_type_errors;
use crate::symbols::extract_symbols;
use crate::dependencies::{extract_dependencies, extract_imports};
use crate::visitors::dependency_visitor::ImportInfo;
use crate::tests::extract_tests;

#[derive(Default, Clone)]
pub struct AnalysisOptions {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub parallel: bool,
    pub exported_only: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct FileDependency {
    pub file: String,
    pub imports: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct FileImports {
    pub file: String,
    pub imports: Vec<ImportInfo>,
}

#[derive(Debug, serde::Serialize)]
pub struct SymbolDependency {
    pub symbol: String,
    pub source_file: String,
    pub used_in: Vec<String>,
}

#[derive(Default, Debug, serde::Serialize)]
pub struct AnalysisResult {
    pub type_errors: Vec<TypeError>,
    pub symbols: Vec<SymbolInfo>,
    pub dependencies: Vec<FileDependency>,
    pub file_imports: Vec<FileImports>,
    pub tests: Vec<TypeTest>,
    pub total_files: usize,
}

pub struct FileAnalysis {
    pub file_path: PathBuf,
    pub type_errors: Vec<TypeError>,
    pub symbols: Vec<SymbolInfo>,
    pub dependencies: Vec<String>,
    pub imports: Vec<ImportInfo>,
    pub tests: Vec<TypeTest>,
}

pub struct Analyzer {
    options: AnalysisOptions,
}

impl Analyzer {
    pub fn new(options: AnalysisOptions) -> Self {
        Self { options }
    }

    pub fn analyze_files(&self, files: &[PathBuf]) -> Result<AnalysisResult> {
        let mut result = AnalysisResult {
            total_files: files.len(),
            ..Default::default()
        };

        let file_analyses: Vec<FileAnalysis> = if self.options.parallel {
            files.par_iter()
                .filter_map(|path| self.analyze_single_file(path).ok())
                .collect()
        } else {
            files.iter()
                .filter_map(|path| self.analyze_single_file(path).ok())
                .collect()
        };

        for file_analysis in file_analyses {
            result.type_errors.extend(file_analysis.type_errors);
            result.symbols.extend(file_analysis.symbols);

            // Preserve file context for dependencies
            if !file_analysis.dependencies.is_empty() {
                result.dependencies.push(FileDependency {
                    file: file_analysis.file_path.to_string_lossy().to_string(),
                    imports: file_analysis.dependencies,
                });
            }

            // Preserve imports with symbols
            if !file_analysis.imports.is_empty() {
                result.file_imports.push(FileImports {
                    file: file_analysis.file_path.to_string_lossy().to_string(),
                    imports: file_analysis.imports,
                });
            }

            result.tests.extend(file_analysis.tests);
        }

        Ok(result)
    }

    pub fn analyze_single_file(&self, path: &Path) -> Result<FileAnalysis> {
        let source_code = std::fs::read_to_string(path)?;
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).map_err(|_| Error::InvalidSourceType(path.to_string_lossy().to_string()))?;
        
        let parser = Parser::new(&allocator, &source_code, source_type);
        let parse_ret = parser.parse();

        let semantic_ret = SemanticBuilder::new(&source_code).build(&parse_ret.program);
        let semantic = semantic_ret.semantic;
        let diagnostics = semantic_ret.errors;
        
        let file_path_str = path.to_string_lossy().to_string();

        let type_errors = extract_type_errors(&source_code, &semantic, &diagnostics, &parse_ret.program, file_path_str.clone());
        let symbols = extract_symbols(&source_code, &parse_ret.program, file_path_str.clone(), self.options.exported_only);
        let dependencies = extract_dependencies(&parse_ret.program, path.to_path_buf());
        let imports = extract_imports(&parse_ret.program, path.to_path_buf());
        let tests = extract_tests(&parse_ret.program, file_path_str);

        Ok(FileAnalysis {
            file_path: path.to_path_buf(),
            type_errors,
            symbols,
            dependencies,
            imports,
            tests,
        })
    }
}