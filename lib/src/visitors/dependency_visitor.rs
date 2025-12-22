use oxc_ast::visit::{walk, Visit};
use oxc_ast::ast::*;
use std::path::PathBuf;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ImportInfo {
    pub source: String,
    pub symbols: Vec<String>,
}

pub struct DependencyVisitor {
    pub dependencies: Vec<String>,
    pub imports: Vec<ImportInfo>,
    pub current_file: PathBuf,
}

impl DependencyVisitor {
    pub fn new(current_file: PathBuf) -> Self {
        Self {
            dependencies: Vec::new(),
            imports: Vec::new(),
            current_file,
        }
    }
}

impl<'a> Visit<'a> for DependencyVisitor {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let source = decl.source.value.to_string();
        self.dependencies.push(source.clone());

        // Extract imported symbols
        let mut symbols = Vec::new();

        if let Some(specifiers) = &decl.specifiers {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        // Named import: import { foo } from './bar'
                        symbols.push(spec.local.name.to_string());
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        // Default import: import foo from './bar'
                        symbols.push(spec.local.name.to_string());
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        // Namespace import: import * as foo from './bar'
                        symbols.push(format!("* as {}", spec.local.name));
                    }
                }
            }
        }

        if !symbols.is_empty() {
            self.imports.push(ImportInfo { source, symbols });
        }

        walk::walk_import_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            let source_str = source.value.to_string();
            self.dependencies.push(source_str.clone());

            // Extract re-exported symbols
            let mut symbols = Vec::new();
            for spec in &decl.specifiers {
                symbols.push(spec.local.name().to_string());
            }

            if !symbols.is_empty() {
                self.imports.push(ImportInfo {
                    source: source_str,
                    symbols,
                });
            }
        }
        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let source = decl.source.value.to_string();
        self.dependencies.push(source.clone());

        // Export * means all symbols
        self.imports.push(ImportInfo {
            source,
            symbols: vec!["*".to_string()],
        });

        walk::walk_export_all_declaration(self, decl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn parse_and_visit(source: &str) -> Vec<String> {
        let allocator = Allocator::default();
        let source_type = SourceType::default().with_typescript(true);
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut visitor = DependencyVisitor::new(PathBuf::from("test.ts"));
        visitor.visit_program(&ret.program);
        
        visitor.dependencies
    }

    #[test]
    fn test_import_declaration() {
        let source = "import { x } from './utils';";
        let deps = parse_and_visit(source);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "./utils");
    }

    #[test]
    fn test_export_named_from() {
        let source = "export { x } from './lib';";
        let deps = parse_and_visit(source);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "./lib");
    }

    #[test]
    fn test_export_all_from() {
        let source = "export * from './all';";
        let deps = parse_and_visit(source);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "./all");
    }

    #[test]
    fn test_mixed_deps() {
        let source = r#"
            import a from 'pkg1';
            import b from 'pkg2';
            export * from 'pkg3';
        "#;
        let deps = parse_and_visit(source);
        assert_eq!(deps.len(), 3);
        assert!(deps.contains(&"pkg1".to_string()));
        assert!(deps.contains(&"pkg2".to_string()));
        assert!(deps.contains(&"pkg3".to_string()));
    }
}