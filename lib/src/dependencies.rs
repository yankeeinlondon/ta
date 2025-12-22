use std::path::{Path, PathBuf};
use oxc_ast::ast::Program;
use oxc_ast::visit::Visit;
use crate::visitors::dependency_visitor::{DependencyVisitor, ImportInfo};

pub fn extract_dependencies(
    program: &Program<'_>,
    file_path: PathBuf,
) -> Vec<String> {
    let mut visitor = DependencyVisitor::new(file_path);
    visitor.visit_program(program);
    visitor.dependencies
}

pub fn extract_imports(
    program: &Program<'_>,
    file_path: PathBuf,
) -> Vec<ImportInfo> {
    let mut visitor = DependencyVisitor::new(file_path);
    visitor.visit_program(program);
    visitor.imports
}

/// Resolve a TypeScript import specifier to an actual file path
pub fn resolve_import_path(import_specifier: &str, importing_file: &Path) -> Option<PathBuf> {
    // External packages (don't start with . or /)
    if !import_specifier.starts_with('.') && !import_specifier.starts_with('/') {
        return None;
    }

    let importing_dir = importing_file.parent()?;
    let base_path = importing_dir.join(import_specifier);

    // Try various extensions
    let extensions = [".ts", ".tsx", ".d.ts"];

    // Try with exact name first (already has extension)
    if base_path.exists() {
        return Some(normalize_path(&base_path));
    }

    // Try adding extensions
    for ext in extensions {
        let mut candidate = base_path.clone();
        let file_name = candidate.file_name()?.to_string_lossy().to_string();
        candidate.set_file_name(format!("{}{}", file_name, ext));

        if candidate.exists() {
            return Some(normalize_path(&candidate));
        }
    }

    // Try index files in directory
    for ext in extensions {
        let candidate = base_path.join(format!("index{}", ext));
        if candidate.exists() {
            return Some(normalize_path(&candidate));
        }
    }

    None
}

/// Normalize path by removing redundant . and .. components
fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {
                // Skip '.' components unless it's the first component
                if components.is_empty() {
                    components.push(component);
                }
            }
            Component::ParentDir => {
                // Handle '..' by popping previous component if possible
                if let Some(Component::Normal(_)) = components.last() {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            _ => components.push(component),
        }
    }

    components.iter().collect()
}
