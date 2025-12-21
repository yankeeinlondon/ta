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

    fn add_symbol(&mut self, name: String, kind: SymbolKind, span: Span, params: Option<Vec<ParameterInfo>>, props: Option<Vec<PropertyInfo>>) {
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
        });
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
                 let param_name = match &param.pattern.kind {
                     BindingPatternKind::BindingIdentifier(id) => id.name.to_string(),
                     _ => "complex_pattern".to_string(),
                 };
                 let type_ann = param.pattern.type_annotation.as_ref().map(|_t| {
                     "type".to_string() 
                 });
                 params.push(ParameterInfo { name: param_name, type_annotation: type_ann });
            }

            self.add_symbol(name, SymbolKind::Function, func.span, Some(params), None);
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
                                 type_annotation: prop.type_annotation.as_ref().map(|_| "type".to_string()),
                             });
                        }
                    }
                    ClassElement::MethodDefinition(method) => {
                        if let PropertyKey::StaticIdentifier(key) = &method.key {
                             props.push(PropertyInfo {
                                 name: format!("{}()", key.name),
                                 type_annotation: None,
                             });
                        }
                    }
                    _ => {}
                }
            }
            
            self.add_symbol(name, SymbolKind::Class, class.span, None, Some(props));
        }

        walk::walk_class(self, class);
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
             self.add_symbol(id.name.to_string(), SymbolKind::Variable, decl.span, None, None);
        }
        walk::walk_variable_declarator(self, decl);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
        let name = decl.id.name.to_string();
        self.add_symbol(name, SymbolKind::Interface, decl.span, None, None);
        walk::walk_ts_interface_declaration(self, decl);
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
        let name = decl.id.name.to_string();
        self.add_symbol(name, SymbolKind::Type, decl.span, None, None);
        walk::walk_ts_type_alias_declaration(self, decl);
    }

    fn visit_ts_enum_declaration(&mut self, decl: &TSEnumDeclaration<'a>) {
        let name = decl.id.name.to_string();
        self.add_symbol(name, SymbolKind::Enum, decl.span, None, None);
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