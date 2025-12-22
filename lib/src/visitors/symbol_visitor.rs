use crate::models::{SymbolInfo, SymbolKind, ParameterInfo, PropertyInfo};
use oxc_ast::visit::{walk, Visit};
use oxc_ast::ast::*;
use oxc_span::Span;
use oxc_semantic::ScopeFlags;

pub struct SymbolVisitor<'a> {
    pub symbols: Vec<SymbolInfo>,
    pub exported_only: bool,
    pub source: &'a str,
    file_path: String,
    is_exporting: bool,
}

impl<'a> SymbolVisitor<'a> {
    pub fn new(source: &'a str, file_path: String, exported_only: bool) -> Self {
        Self {
            symbols: Vec::new(),
            exported_only,
            source,
            file_path,
            is_exporting: false,
        }
    }

    fn add_symbol(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: Span,
        params: Option<Vec<ParameterInfo>>,
        props: Option<Vec<PropertyInfo>>,
        return_type: Option<String>,
        jsdoc: Option<String>,
    ) {
        if self.exported_only && !self.is_exporting {
            return;
        }

        let (start_line, _) = self.get_line_col(span.start);
        let (end_line, _) = self.get_line_col(span.end);

        self.symbols.push(SymbolInfo {
            name,
            kind,
            file: self.file_path.clone(),
            start_line,
            end_line,
            exported: self.is_exporting,
            parameters: params,
            properties: props,
            return_type,
            jsdoc,
        });
    }

    /// Extract JSDoc comment from leading comments
    fn extract_jsdoc(&self, span: Span) -> Option<String> {
        // Look backwards from span.start to find JSDoc comment
        let start = span.start as usize;
        if start == 0 {
            return None;
        }

        let before = &self.source[..start];

        // Find JSDoc block /** ... */ immediately before this declaration
        let trimmed = before.trim_end();
        if trimmed.ends_with("*/") {
            if let Some(doc_start) = trimmed.rfind("/**") {
                let doc = &trimmed[doc_start..];
                // Clean up the JSDoc: remove /** */, strip * from each line
                let cleaned = doc.lines()
                    .map(|line| {
                        line.trim()
                            .trim_start_matches("/**")
                            .trim_start_matches("*/")
                            .trim_start_matches('*')
                            .trim()
                    })
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");

                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }

        None
    }

    fn get_line_col(&self, offset: u32) -> (usize, usize) {
        let offset = offset as usize;
        if offset >= self.source.len() {
            return (1, 1);
        }
        let before = &self.source[..offset];
        let line = before.lines().count().max(1);
        (line, 0)
    }

    /// Extract parameter name from binding pattern (handles defaults and destructuring)
    fn extract_param_name(pattern: &BindingPattern) -> String {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(id) => id.name.to_string(),
            BindingPatternKind::ObjectPattern(_) => "{...}".to_string(),
            BindingPatternKind::ArrayPattern(_) => "[...]".to_string(),
            BindingPatternKind::AssignmentPattern(assign) => {
                // Parameter with default value
                Self::extract_param_name(&assign.left)
            }
        }
    }

    /// Extract type annotation from binding pattern (handles defaults)
    fn extract_type_annotation(&self, pattern: &BindingPattern) -> Option<String> {
        match &pattern.kind {
            BindingPatternKind::AssignmentPattern(assign) => {
                // For parameters with defaults, get type from the left side
                Self::extract_type_annotation(self, &assign.left)
            }
            _ => {
                // For regular patterns, get type annotation directly
                pattern.type_annotation.as_ref().map(|t| {
                    let span = t.span;
                    self.source.get(span.start as usize..span.end as usize)
                        .unwrap_or("type")
                        .trim_start_matches(':')
                        .trim()
                        .to_string()
                })
            }
        }
    }
}

impl<'a> Visit<'a> for SymbolVisitor<'a> {
    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        let was_exporting = self.is_exporting;
        self.is_exporting = true;
        walk::walk_export_named_declaration(self, decl);
        self.is_exporting = was_exporting;
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        let was_exporting = self.is_exporting;
        self.is_exporting = true;
        walk::walk_export_default_declaration(self, decl);
        self.is_exporting = was_exporting;
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        let name = func.id.as_ref().map(|id| id.name.to_string());

        if let Some(name) = name {
            let mut params = Vec::new();
            for param in &func.params.items {
                 // Extract parameter name (handle both simple and complex patterns)
                 let param_name = Self::extract_param_name(&param.pattern);

                 // Extract type annotation (handles defaults)
                 let type_ann = self.extract_type_annotation(&param.pattern);

                 params.push(ParameterInfo {
                     name: param_name,
                     type_annotation: type_ann,
                     description: None,
                 });
            }

            // Extract return type
            let return_type = func.return_type.as_ref().map(|rt| {
                let span = rt.span;
                self.source.get(span.start as usize..span.end as usize)
                    .unwrap_or("unknown")
                    .trim_start_matches(':')
                    .trim()
                    .to_string()
            });

            // Extract JSDoc
            let jsdoc = self.extract_jsdoc(func.span);

            self.add_symbol(name, SymbolKind::Function, func.span, Some(params), None, return_type, jsdoc);
        }

        walk::walk_function(self, func, flags);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        let name = class.id.as_ref().map(|id| id.name.to_string());

        if let Some(name) = name {
            let mut props = Vec::new();
            for element in &class.body.body {
                match element {
                    ClassElement::PropertyDefinition(prop) => {
                        if let PropertyKey::StaticIdentifier(key) = &prop.key {
                             props.push(PropertyInfo {
                                 name: key.name.to_string(),
                                 type_annotation: prop.type_annotation.as_ref().map(|t| {
                                     let span = t.span;
                                     self.source.get(span.start as usize..span.end as usize)
                                         .unwrap_or("type")
                                         .trim_start_matches(':')
                                         .trim()
                                         .to_string()
                                 }),
                                 description: None,
                             });
                        }
                    }
                    ClassElement::MethodDefinition(method) => {
                        if let PropertyKey::StaticIdentifier(key) = &method.key {
                             props.push(PropertyInfo {
                                 name: format!("{}()", key.name),
                                 type_annotation: None,
                                 description: None,
                             });
                        }
                    }
                    _ => {}
                }
            }

            let jsdoc = self.extract_jsdoc(class.span);
            self.add_symbol(name, SymbolKind::Class, class.span, None, Some(props), None, jsdoc);
        }

        walk::walk_class(self, class);
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
             let jsdoc = self.extract_jsdoc(decl.span);
             self.add_symbol(id.name.to_string(), SymbolKind::Variable, decl.span, None, None, None, jsdoc);
        }
        walk::walk_variable_declarator(self, decl);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
        let name = decl.id.name.to_string();

        // Extract interface properties
        let mut props = Vec::new();
        for element in &decl.body.body {
            match element {
                TSSignature::TSPropertySignature(prop) => {
                    if let PropertyKey::StaticIdentifier(key) = &prop.key {
                        let type_ann = prop.type_annotation.as_ref().map(|t| {
                            let span = t.span;
                            self.source.get(span.start as usize..span.end as usize)
                                .unwrap_or("type")
                                .trim_start_matches(':')
                                .trim()
                                .to_string()
                        });
                        props.push(PropertyInfo {
                            name: key.name.to_string(),
                            type_annotation: type_ann,
                            description: None,
                        });
                    }
                }
                _ => {}
            }
        }

        let jsdoc = self.extract_jsdoc(decl.span);
        self.add_symbol(name, SymbolKind::Interface, decl.span, None, Some(props), None, jsdoc);
        walk::walk_ts_interface_declaration(self, decl);
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
        let name = decl.id.name.to_string();
        let jsdoc = self.extract_jsdoc(decl.span);
        self.add_symbol(name, SymbolKind::Type, decl.span, None, None, None, jsdoc);
        walk::walk_ts_type_alias_declaration(self, decl);
    }

    fn visit_ts_enum_declaration(&mut self, decl: &TSEnumDeclaration<'a>) {
        let name = decl.id.name.to_string();
        let jsdoc = self.extract_jsdoc(decl.span);
        self.add_symbol(name, SymbolKind::Enum, decl.span, None, None, None, jsdoc);
        walk::walk_ts_enum_declaration(self, decl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn parse_and_visit(source: &str, exported_only: bool) -> Vec<SymbolInfo> {
        let allocator = Allocator::default();
        let source_type = SourceType::default().with_typescript(true);
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut visitor = SymbolVisitor::new(source, "test.ts".to_string(), exported_only);
        visitor.visit_program(&ret.program);
        
        visitor.symbols
    }

    #[test]
    fn test_extract_function() {
        let source = "function foo(a: number) {}";
        let symbols = parse_and_visit(source, false);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "foo");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[0].parameters.as_ref().unwrap().len(), 1);
        assert_eq!(symbols[0].parameters.as_ref().unwrap()[0].name, "a");
    }

    #[test]
    fn test_extract_class() {
        let source = "class MyClass { prop: string; method() {} }";
        let symbols = parse_and_visit(source, false);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MyClass");
        assert_eq!(symbols[0].kind, SymbolKind::Class);
        let props = symbols[0].properties.as_ref().unwrap();
        assert_eq!(props.len(), 2);
    }

    #[test]
    fn test_extract_variable() {
        let source = "const x = 1;";
        let symbols = parse_and_visit(source, false);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "x");
        assert_eq!(symbols[0].kind, SymbolKind::Variable);
    }

    #[test]
    fn test_extract_exported_only() {
        let source = "const x = 1; export const y = 2;";
        let symbols = parse_and_visit(source, true);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "y");
    }

    #[test]
    fn test_extract_interface_and_type() {
        let source = "interface I {} type T = {};";
        let symbols = parse_and_visit(source, false);
        assert_eq!(symbols.len(), 2);
        assert!(symbols.iter().any(|s| s.name == "I" && s.kind == SymbolKind::Interface));
        assert!(symbols.iter().any(|s| s.name == "T" && s.kind == SymbolKind::Type));
    }
}